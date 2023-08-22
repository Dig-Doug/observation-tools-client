use crate::builders::NumberBuilder;
use crate::builders::Point2Builder;
use crate::builders::Point3Builder;
use crate::builders::Transform3Builder;
use crate::builders::Vector2Builder;
use crate::builders::Vector3Builder;
use nalgebra::Matrix3;
use nalgebra::Matrix4;
use nalgebra::Point2;
use nalgebra::Point3;
use nalgebra::RealField;
use nalgebra::Rotation;
use nalgebra::Scalar;
use nalgebra::Similarity;
use nalgebra::Transform3;
use nalgebra::Translation;
use nalgebra::Vector2;
use nalgebra::Vector3;

impl<T: Scalar + Into<NumberBuilder>> Into<Point2Builder> for Point2<T> {
    fn into(self) -> Point2Builder {
        Point2Builder::new(self.x.clone().into(), self.y.clone().into())
    }
}

impl<T: Scalar + Into<NumberBuilder>> Into<Point3Builder> for Point3<T> {
    fn into(self) -> Point3Builder {
        Point3Builder::new(
            self.x.clone().into(),
            self.y.clone().into(),
            self.z.clone().into(),
        )
    }
}

impl<T: Scalar + Into<NumberBuilder>> Into<Vector2Builder> for Vector2<T> {
    fn into(self) -> Vector2Builder {
        Vector2Builder::new(self.x.clone().into(), self.y.clone().into())
    }
}

impl<T: Scalar + Into<NumberBuilder>> Into<Vector3Builder> for Vector3<T> {
    fn into(self) -> Vector3Builder {
        Vector3Builder::new(
            self.x.clone().into(),
            self.y.clone().into(),
            self.z.clone().into(),
        )
    }
}

impl<T: Scalar + RealField + Into<NumberBuilder>> Into<Transform3Builder> for Transform3<T> {
    fn into(self) -> Transform3Builder {
        Transform3Builder {
            proto: transform_to_transform3_proto(&self),
        }
    }
}

pub fn transform_to_transform3_proto<T: Scalar + RealField + Into<NumberBuilder>>(
    transform: &Transform3<T>,
) -> crate::generated::Transform3 {
    let mut proto = crate::generated::Transform3::new();
    *proto.mut_matrix() = matrix_to_matrix4x4_proto(transform.matrix());
    proto
}

pub fn transform3_proto_to_transform(proto: &crate::generated::Transform3) -> Transform3<f32> {
    match proto.data.as_ref().unwrap() {
        crate::generated::transform3::Data::Trs(trs) => nalgebra::convert(Similarity::from_parts(
            Translation::from(
                trs.translation
                    .as_ref()
                    .map(|v| point3_proto_to_point3(&v))
                    .unwrap_or(Point3::origin()),
            ),
            Rotation::identity(),
            1.0,
        )),
        crate::generated::transform3::Data::Matrix(matrix) => {
            Transform3::<f32>::from_matrix_unchecked(matrix4x4_proto_to_matrix(matrix))
        }
        crate::generated::transform3::Data::Identity(_) => Transform3::identity(),
        _ => panic!("Unhandled transform3 case"),
    }
}

pub fn number_to_proto<T: Scalar + RealField + Into<NumberBuilder>>(
    value: T,
) -> crate::generated::Number {
    let b: NumberBuilder = value.into();
    b.proto
}

pub fn number_proto_to_f32(proto: &crate::generated::Number) -> f32 {
    proto.d as f32
}

pub fn point2_proto_to_point2(proto: &crate::generated::Point2) -> Point2<f32> {
    Point2::new(number_proto_to_f32(&proto.x), number_proto_to_f32(&proto.y))
}

pub fn point2_proto_to_point3(proto: &crate::generated::Point2) -> Point3<f32> {
    Point3::new(
        number_proto_to_f32(&proto.x),
        number_proto_to_f32(&proto.y),
        0.0,
    )
}

pub fn point3_proto_to_point3(proto: &crate::generated::Point3) -> Point3<f32> {
    Point3::new(
        number_proto_to_f32(&proto.x),
        number_proto_to_f32(&proto.y),
        number_proto_to_f32(&proto.z),
    )
}

pub fn vector2_proto_to_vector2(proto: &crate::generated::Vector2) -> Vector2<f32> {
    Vector2::new(number_proto_to_f32(&proto.x), number_proto_to_f32(&proto.y))
}

pub fn vector3_proto_to_vector3(proto: &crate::generated::Vector3) -> Vector3<f32> {
    Vector3::new(
        number_proto_to_f32(&proto.x),
        number_proto_to_f32(&proto.y),
        number_proto_to_f32(&proto.z),
    )
}

pub fn matrix3x3_proto_to_matrix(proto: &crate::generated::Matrix3x3) -> Matrix3<f32> {
    Matrix3::new(
        number_proto_to_f32(&proto.m0_0),
        number_proto_to_f32(&proto.m0_1),
        number_proto_to_f32(&proto.m0_2),
        number_proto_to_f32(&proto.m1_0),
        number_proto_to_f32(&proto.m1_1),
        number_proto_to_f32(&proto.m1_2),
        number_proto_to_f32(&proto.m2_0),
        number_proto_to_f32(&proto.m2_1),
        number_proto_to_f32(&proto.m2_2),
    )
}

pub fn matrix4x4_proto_to_matrix(proto: &crate::generated::Matrix4x4) -> Matrix4<f32> {
    Matrix4::new(
        number_proto_to_f32(&proto.m0_0),
        number_proto_to_f32(&proto.m0_1),
        number_proto_to_f32(&proto.m0_2),
        number_proto_to_f32(&proto.m0_3),
        number_proto_to_f32(&proto.m1_0),
        number_proto_to_f32(&proto.m1_1),
        number_proto_to_f32(&proto.m1_2),
        number_proto_to_f32(&proto.m1_3),
        number_proto_to_f32(&proto.m2_0),
        number_proto_to_f32(&proto.m2_1),
        number_proto_to_f32(&proto.m2_2),
        number_proto_to_f32(&proto.m2_3),
        number_proto_to_f32(&proto.m3_0),
        number_proto_to_f32(&proto.m3_1),
        number_proto_to_f32(&proto.m3_2),
        number_proto_to_f32(&proto.m3_3),
    )
}

pub fn matrix_to_matrix4x4_proto<T: Scalar + RealField + Into<NumberBuilder>>(
    matrix: &Matrix4<T>,
) -> crate::generated::Matrix4x4 {
    let mut proto = crate::generated::Matrix4x4::new();
    proto.m0_0 = Some(number_to_proto(matrix.m11.clone())).into();
    proto.m0_1 = Some(number_to_proto(matrix.m12.clone())).into();
    proto.m0_2 = Some(number_to_proto(matrix.m13.clone())).into();
    proto.m0_3 = Some(number_to_proto(matrix.m14.clone())).into();
    proto.m1_0 = Some(number_to_proto(matrix.m21.clone())).into();
    proto.m1_1 = Some(number_to_proto(matrix.m22.clone())).into();
    proto.m1_2 = Some(number_to_proto(matrix.m23.clone())).into();
    proto.m1_3 = Some(number_to_proto(matrix.m24.clone())).into();
    proto.m2_0 = Some(number_to_proto(matrix.m31.clone())).into();
    proto.m2_1 = Some(number_to_proto(matrix.m32.clone())).into();
    proto.m2_2 = Some(number_to_proto(matrix.m33.clone())).into();
    proto.m2_3 = Some(number_to_proto(matrix.m34.clone())).into();
    proto.m3_0 = Some(number_to_proto(matrix.m41.clone())).into();
    proto.m3_1 = Some(number_to_proto(matrix.m42.clone())).into();
    proto.m3_2 = Some(number_to_proto(matrix.m43.clone())).into();
    proto.m3_3 = Some(number_to_proto(matrix.m44.clone())).into();
    proto
}

pub fn transform2_proto_to_transform(proto: &crate::generated::Transform2) -> Transform3<f32> {
    match proto.data.as_ref().unwrap() {
        crate::generated::transform2::Data::Trs(trs) => {
            let t = trs
                .translation
                .as_ref()
                .map(|v| point2_proto_to_point2(&v))
                .unwrap_or(Point2::origin());
            nalgebra::convert(Similarity::from_parts(
                Translation::from(Vector3::new(t[0], t[1], 0.0)),
                Rotation::identity(),
                1.0,
            ))
        }
        crate::generated::transform2::Data::Identity(_) => Transform3::identity(),
        _ => panic!("Unhandled transform2 case"),
    }
}
