use super::*;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Shape {
    Circle { radius: Coord },
    Rectangle { width: Coord, height: Coord },
    RectangleOutline { width: Coord, height: Coord },
}

impl Shape {
    pub fn circle(radius: Coord) -> Self {
        Self::Circle { radius }
    }

    pub fn rectangle(size: vec2<Coord>) -> Self {
        Self::Rectangle {
            width: size.x,
            height: size.y,
        }
    }

    pub fn rectangle_outline(size: vec2<Coord>) -> Self {
        Self::RectangleOutline {
            width: size.x,
            height: size.y,
        }
    }

    pub fn to_parry(self) -> Box<dyn parry2d::shape::Shape> {
        match self {
            Shape::Circle { radius } => Box::new(parry2d::shape::Ball::new(radius.as_f32())),
            Shape::Rectangle { width, height } => {
                if width == R32::ZERO || height == R32::ZERO {
                    return Box::new(parry2d::shape::Ball::new(0.0));
                }
                let aabb = Aabb2::ZERO.extend_symmetric(vec2(width, height).as_f32() / 2.0);
                let points = aabb.corners().map(|p| {
                    let vec2(x, y) = p;
                    parry2d::math::Point::new(x, y)
                });
                match parry2d::shape::ConvexPolygon::from_convex_hull(&points) {
                    Some(shape) => Box::new(shape),
                    None => Box::new(parry2d::shape::Ball::new(0.0)),
                }
            }
            Shape::RectangleOutline { width, height } => {
                let aabb = Aabb2::ZERO.extend_symmetric(vec2(width, height).as_f32() / 2.0);
                let vertices = [
                    aabb.bottom_left(),
                    aabb.bottom_right(),
                    aabb.top_right(),
                    aabb.top_left(),
                    aabb.bottom_left(),
                ]
                .map(|vec2(x, y)| parry2d::math::Point::new(x, y));
                Box::new(parry2d::shape::Polyline::new(vertices.to_vec(), None))
            }
        }
    }

    pub fn scaled(self, scale: Coord) -> Self {
        match self {
            Shape::Circle { radius } => Shape::Circle {
                radius: radius * scale,
            },
            Shape::Rectangle { width, height } => Shape::Rectangle {
                width: width * scale,
                height: height * scale,
            },
            Shape::RectangleOutline { width, height } => Shape::RectangleOutline {
                width: width * scale,
                height: height * scale,
            },
        }
    }
}
