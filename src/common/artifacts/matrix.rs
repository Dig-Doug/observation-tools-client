use crate::artifacts::Number;
use serde::Deserialize;
use serde::Serialize;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Matrix3x3 {
    // m<Row>_<Col>
    pub m0_0: Number,
    pub m0_1: Number,
    pub m0_2: Number,
    pub m1_0: Number,
    pub m1_1: Number,
    pub m1_2: Number,
    pub m2_0: Number,
    pub m2_1: Number,
    pub m2_2: Number,
}

impl Into<nalgebra::Matrix3<f64>> for Matrix3x3 {
    fn into(self) -> nalgebra::Matrix3<f64> {
        nalgebra::Matrix3::new(
            self.m0_0.into(),
            self.m0_1.into(),
            self.m0_2.into(),
            self.m1_0.into(),
            self.m1_1.into(),
            self.m1_2.into(),
            self.m2_0.into(),
            self.m2_1.into(),
            self.m2_2.into(),
        )
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Matrix4x4 {
    // m<Row>_<Col>
    pub m0_0: Number,
    pub m0_1: Number,
    pub m0_2: Number,
    pub m0_3: Number,
    pub m1_0: Number,
    pub m1_1: Number,
    pub m1_2: Number,
    pub m1_3: Number,
    pub m2_0: Number,
    pub m2_1: Number,
    pub m2_2: Number,
    pub m2_3: Number,
    pub m3_0: Number,
    pub m3_1: Number,
    pub m3_2: Number,
    pub m3_3: Number,
}

impl<T: nalgebra::Scalar + Into<Number>> Into<Matrix4x4> for &nalgebra::Matrix4<T> {
    fn into(self) -> Matrix4x4 {
        // TODO(doug): Not sure why we need to clone... so made it for &Matrix4.
        Matrix4x4 {
            m0_0: self.m11.clone().into(),
            m0_1: self.m12.clone().into(),
            m0_2: self.m13.clone().into(),
            m0_3: self.m14.clone().into(),
            m1_0: self.m21.clone().into(),
            m1_1: self.m22.clone().into(),
            m1_2: self.m23.clone().into(),
            m1_3: self.m24.clone().into(),
            m2_0: self.m31.clone().into(),
            m2_1: self.m32.clone().into(),
            m2_2: self.m33.clone().into(),
            m2_3: self.m34.clone().into(),
            m3_0: self.m41.clone().into(),
            m3_1: self.m42.clone().into(),
            m3_2: self.m43.clone().into(),
            m3_3: self.m44.clone().into(),
        }
    }
}

impl Into<nalgebra::Matrix4<f64>> for Matrix4x4 {
    fn into(self) -> nalgebra::Matrix4<f64> {
        nalgebra::Matrix4::new(
            self.m0_0.into(),
            self.m0_1.into(),
            self.m0_2.into(),
            self.m0_3.into(),
            self.m1_0.into(),
            self.m1_1.into(),
            self.m1_2.into(),
            self.m1_3.into(),
            self.m2_0.into(),
            self.m2_1.into(),
            self.m2_2.into(),
            self.m2_3.into(),
            self.m3_0.into(),
            self.m3_1.into(),
            self.m3_2.into(),
            self.m3_3.into(),
        )
    }
}
