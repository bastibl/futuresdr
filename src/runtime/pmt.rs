use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PMT {
    Null,
    String(String),
    U32(u32),
    Double(f64),
    VecF32(Vec<f32>),
    Blob(Vec<u8>),
}

impl PMT {

    pub fn is_string(&self) -> bool {
        match self {
            PMT::String(_) => true,
            _ => false
        }
    }

    pub fn to_string(&self) -> Option<String> {
        match self {
            PMT::String(s) => Some(s.clone()),
            _ => None
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn pmt() {
        let p = PMT::Null;
        assert_eq!(p.is_string(), false);
        assert_eq!(p.to_string(), None);
        let p = PMT::String("foo".to_owned());
        assert_eq!(p.is_string(), true);
        assert_eq!(p.to_string(), Some("foo".to_owned()));
    }

    #[test]
    fn pmt_serde() {
        let p = PMT::Null;
        let mut s = flexbuffers::FlexbufferSerializer::new();
        p.serialize(&mut s).unwrap();

        let r = flexbuffers::Reader::get_root(s.view()).unwrap();
        let p2 = PMT::deserialize(r).unwrap();

        assert_eq!(p, p2);
    }

    #[test]
    fn pmt_eq() {
        let a = PMT::Null;
        let b = PMT::U32(123);
        assert_ne!(a, b);

        let c = PMT::Null;
        let d = PMT::U32(12);
        let e = PMT::U32(123);
        assert_eq!(a, c);
        assert_eq!(b, e);
        assert_ne!(b, d);
    }
}
 
