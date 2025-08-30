use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{glib, gsk};

use crate::classify;

/// Wrapper type around `Vec<classify::Stroke>` to allow using it in signals.
#[derive(Clone, Debug, glib::Boxed)]
#[boxed_type(name = "BoxedStrokes")]
pub struct BoxedStrokes(pub Vec<classify::Stroke>);

mod imp {
    use adw::subclass::bin::BinImpl;
    use glib::subclass::Signal;
    use gtk::gdk;
    use itertools::Itertools;

    use crate::classify;

    use super::*;
    use std::{cell::RefCell, sync::OnceLock};

    #[derive(Default, Debug, gtk::CompositeTemplate)]
    #[template(resource = "/io/github/finefindus/Hieroglyphic/ui/drawing-area.ui")]
    pub struct DrawingArea {
        #[template_child]
        drag: TemplateChild<gtk::GestureDrag>,
        pub(super) strokes: RefCell<Vec<classify::Stroke>>,
        pub(super) current_stroke: RefCell<classify::Stroke>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for DrawingArea {
        const NAME: &'static str = "DrawingArea";
        type ParentType = adw::Bin;
        type Type = super::DrawingArea;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for DrawingArea {
        fn signals() -> &'static [Signal] {
            static SIGNALS: OnceLock<Vec<Signal>> = OnceLock::new();
            SIGNALS.get_or_init(|| {
                vec![
                    Signal::builder("stroke-drawn")
                        .param_types([BoxedStrokes::static_type()])
                        .build(),
                ]
            })
        }

        fn constructed(&self) {
            self.parent_constructed();
            self.obj().add_controller(self.drag.get());
        }
    }

    impl WidgetImpl for DrawingArea {
        fn snapshot(&self, snapshot: &gtk::Snapshot) {
            self.parent_snapshot(snapshot);
            let color = self.line_color();

            let curr_stroke = self.current_stroke.borrow().clone();
            for stroke in self
                .strokes
                .borrow()
                .iter()
                .chain(std::iter::once(&curr_stroke))
            {
                tracing::trace!("Drawing: {:?}", stroke);

                let path_builder = gsk::PathBuilder::new();
                for (p, q) in stroke.points().tuple_windows() {
                    path_builder.move_to(p.x as f32, p.y as f32);
                    path_builder.line_to(q.x as f32, q.y as f32);
                }

                if stroke.points().count() == 1 {
                    let point = stroke.points().next().unwrap();
                    path_builder.add_circle(
                        &gtk::graphene::Point::new(point.x as f32, point.y as f32),
                        1.5,
                    );
                }

                let path = path_builder.to_path();
                let stroke = gsk::Stroke::new(3.0);
                stroke.set_line_cap(gsk::LineCap::Round);
                let Some(bounds) = path.stroke_bounds(&stroke) else {
                    continue;
                };
                snapshot.push_stroke(&path, &stroke);
                snapshot.append_color(&color, &bounds);
                snapshot.pop();
            }
        }
    }

    impl BinImpl for DrawingArea {}

    #[gtk::template_callbacks]
    impl DrawingArea {
        /// Returns a theme-specific color for the drawing line.
        fn line_color(&self) -> gdk::RGBA {
            if adw::StyleManager::default().is_dark() {
                // #CCCCCC
                gdk::RGBA::new(0.8, 0.8, 0.8, 1.0)
            } else {
                // adw @dark_2 color
                gdk::RGBA::new(0.37, 0.36, 0.39, 1.0)
            }
        }

        #[template_callback]
        fn on_drag_begin(&self, x: f64, y: f64) {
            tracing::trace!("Drag started at {},{}", x, y);
            self.current_stroke
                .borrow_mut()
                .add_point(classify::Point { x, y });
            self.obj().queue_draw();
        }

        #[template_callback]
        fn on_drag_update(&self, x: f64, y: f64) {
            tracing::trace!("Drag update at {},{}", x, y);
            let mut stroke = self.current_stroke.borrow_mut();
            // x,y refers to movements relative to start coord
            let &classify::Point {
                x: prev_x,
                y: prev_y,
            } = stroke.points().next().unwrap();
            stroke.add_point(classify::Point {
                x: prev_x + x,
                y: prev_y + y,
            });
            self.obj().queue_draw();
        }

        #[template_callback]
        fn on_drag_end(&self, x: f64, y: f64) {
            tracing::trace!("Drag end at {},{}", x, y);
            let stroke = self.current_stroke.take();
            self.strokes.borrow_mut().push(stroke);
            self.obj().queue_draw();

            self.obj().emit_by_name(
                "stroke-drawn",
                &[&BoxedStrokes(self.strokes.borrow().clone())],
            )
        }
    }
}

glib::wrapper! {
    pub struct DrawingArea(ObjectSubclass<imp::DrawingArea>)
    @extends gtk::Widget, adw::Bin,
    @implements gtk::Buildable, gtk::Accessible, gtk::ConstraintTarget;
}

#[gtk::template_callbacks]
impl DrawingArea {
    /// Clears the drawing area.
    #[template_callback]
    pub fn clear(&self) {
        //clear previous strokes
        self.imp().strokes.borrow_mut().clear();
        self.imp().current_stroke.borrow_mut().clear();

        self.queue_draw();
    }
}
