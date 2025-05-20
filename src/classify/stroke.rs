use super::{point::Point, rect::Rect};
use itertools::Itertools;
use serde::Serialize;
use std::collections::VecDeque;

// Original code from:
// https://github.com/FineFindus/detexify-rust/blob/311002feb0519f483ef1f9cc8206648286128ff5/src/stroke.rs

/// A list of connectect [`Point`]s.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct Stroke(Vec<Point>);

impl Stroke {
    /// Create a new [`Stoke`] with the given points.
    pub const fn new(points: Vec<Point>) -> Self {
        Self(points)
    }

    /// Returns true if the stroke does not contain any [`Point`]s.
    pub(super) fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns an iterator over the points of the stroke.
    pub fn points(&self) -> impl Iterator<Item = &Point> {
        self.0.iter()
    }

    /// Clears the stroke, removing all points.
    pub fn clear(&mut self) {
        self.0.clear();
    }

    /// Appends a new [`Point`] to the end of the stroke.
    pub fn add_point(&mut self, point: Point) {
        self.0.push(point)
    }

    /// Computes the total length of the strokes.
    /// The length is defined as the sum of the distance all points.
    pub(super) fn length(&self) -> f64 {
        self.0
            .iter()
            .tuple_windows()
            .fold(0.0, |distance, (p, q)| distance + p.euclidean_distance(q))
    }

    /// Computes a bounding box encompassing all points in the stroke.
    ///
    /// # Panics
    /// Panics if the stroke is empty.
    pub(super) fn bounding_box(&self) -> Rect {
        assert!(!self.is_empty());
        self.0
            .iter()
            .skip(1)
            .fold(Rect::from_point(self.0[0]), |mut bb, point| {
                bb.encompass_point(point);
                bb
            })
    }

    /// Maps all points to fit within the given [`Rect`].
    fn refit(&mut self, rect: Rect) {
        let bb = self.bounding_box();
        let scale_x = if bb.width() == 0.0 {
            1.0
        } else {
            1.0 / bb.width() * rect.width()
        };

        let scale_y = if bb.height() == 0.0 {
            1.0
        } else {
            1.0 / bb.height() * rect.height()
        };

        let trans_x = if bb.width() == 0.0 {
            rect.lower_left.x + 0.5 * rect.width()
        } else {
            rect.lower_left.x
        };

        let trans_y = if bb.height() == 0.0 {
            rect.lower_left.y + 0.5 * rect.height()
        } else {
            rect.lower_left.y
        };

        let trans = Point {
            x: trans_x,
            y: trans_y,
        };

        for point in self.0.iter_mut() {
            *point = (*point - bb.lower_left).scale(scale_x, scale_y) + trans
        }
    }

    /// Maps all points to fit within the given [`Rect`],
    /// whilst keeping the aspect-ration and relative distance between the points the same.
    pub(super) fn aspect_refit(&mut self, target: Rect) {
        let source = self.bounding_box();

        let rect = if source.is_point() {
            let centered = (target.lower_left + target.upper_right) * 0.5;
            Rect::from_point(centered)
        } else {
            let reset = source.lower_left;
            let source_ratio = source.width() / source.height();
            let target_ratio = target.width() / target.height();

            let scale_factor = if source_ratio > target_ratio {
                1.0 / source.width() * target.width()
            } else {
                1.0 / source.height() * target.height()
            };

            let offset = if source_ratio > target_ratio {
                Point {
                    x: 0.0,
                    y: (target.height() - scale_factor * source.height()) / 2.0,
                }
            } else {
                Point {
                    x: (target.width() - scale_factor * source.width()) / 2.0,
                    y: 0.0,
                }
            };

            source.map_points(|p| (p - reset) * scale_factor + (offset + target.lower_left))
        };

        self.refit(rect)
    }

    /// Removes duplicate (points that are nearly identical) [`Point`]s.
    pub(super) fn dedup(&mut self) {
        self.0.dedup_by(|&mut p, &mut q| p.approx_eq(q));
    }

    /// Smooths the stroke.
    ///
    /// This is done by averaging the points of the stroke.
    pub(super) fn smooth(&mut self) {
        let len = self.0.len();
        if len < 3 {
            return;
        }

        let mut smoothed = Vec::with_capacity(len);
        // keep start and end point the same
        smoothed.push(self.0[0]);
        // average triplets of all points
        smoothed.extend(
            self.points()
                .tuple_windows()
                .map(|(&x, &y, &z)| (x + y + z) * 3.0f64.recip()),
        );
        smoothed.push(*self.0.last().unwrap());

        self.0 = smoothed;
    }

    /// Redistribute the points in the stroke to have equal distance to each other.
    pub(super) fn redistribute(&mut self, n: usize) {
        // degenerate cases
        if self.0.len() < 2 {
            return;
        }

        assert!(n > 2);

        let dist = self.length() / (n as f64 - 1.0);
        assert!(dist > 0.0);

        let mut left = dist;

        let mut distributed = Vec::with_capacity(self.0.len());
        let mut work_list: VecDeque<Point> = self.0.iter().cloned().collect();

        distributed.push(self.0[0]);

        let mut i = 100;

        while work_list.len() >= 2 {
            i -= 1;
            if i == 0 {
                break;
            }

            let p = work_list.pop_front().unwrap();
            let q = work_list[0];

            let dir = q - p;
            let d = dir.norm();

            if d < left {
                left -= d;
            } else {
                let ins = p + (dir * (left / d));
                left = dist;
                work_list.push_front(ins);
                distributed.push(ins);
            }
        }

        distributed.extend(work_list);

        self.0 = distributed;
    }

    /// Filter the points to only contain dominant points,
    /// based on the given alpha angle threshold.
    pub(super) fn dominant(&mut self, alpha: f64) {
        if self.0.len() < 3 {
            return;
        }

        let mut new_stroke = Vec::with_capacity(self.0.len());
        new_stroke.push(self.0[0]);
        new_stroke.extend(
            self.points()
                .tuple_windows()
                //TODO: use >= or <=?
                .filter(|&(&p, &q, &r)| p.angle(q, r) <= alpha)
                .map(|(&_p, &q, &_r)| q),
        );
        new_stroke.push(*self.0.last().unwrap());

        self.0 = new_stroke;
    }
}

#[cfg(test)]
mod tests {
    use crate::classify::point::{ONE_POINT, ZERO_POINT};

    use super::*;

    const HALF_POINT: Point = Point { x: 0.5, y: 0.5 };

    #[test]
    fn test_bounding_box() {
        assert_eq!(
            Stroke::new(vec![Point { x: 1.0, y: 1.0 }, Point { x: -1.0, y: -1.0 }]).bounding_box(),
            Rect::new(Point { x: -1.0, y: -1.0 }, Point { x: 1.0, y: 1.0 })
        );
    }

    #[test]
    fn test_refit() {
        let r = Rect::new(ZERO_POINT, ONE_POINT);

        let mut s = Stroke::new(vec![Point { x: -100.0, y: 0.0 }]);
        s.refit(r);
        assert_eq!(s, Stroke::new(vec![HALF_POINT]));

        let mut s = Stroke::new(vec![
            Point { x: -100.0, y: 0.0 },
            Point { x: 0.0, y: 100.0 },
        ]);
        s.refit(r);
        assert_eq!(s, Stroke::new(vec![ZERO_POINT, ONE_POINT]));
    }

    #[test]
    fn test_smooth() {
        let mut s = Stroke::new(vec![
            Point {
                x: 1.2311,
                y: 1.323,
            },
            Point {
                x: 2.121,
                y: 2.4123,
            },
            Point { x: 3.213, y: 3.251 },
            Point {
                x: 1.412,
                y: 4.02441,
            },
        ]);
        s.smooth();

        assert_eq!(
            s,
            Stroke::new(vec![
                Point {
                    x: 1.2311,
                    y: 1.323
                },
                Point {
                    x: 2.1883666666666666,
                    y: 2.3287666666666667
                },
                Point {
                    x: 2.2486666666666664,
                    y: 3.229236666666666
                },
                Point {
                    x: 1.412,
                    y: 4.02441
                }
            ])
        )
    }
}
