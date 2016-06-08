extern crate serde;


mod range {
    use super::serde;
    use map::Range;
    use std;

    impl<T> serde::Serialize for Range<T>
        where T: serde::Serialize,
    {
        fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
            where S: serde::Serializer,
        {
            struct Visitor<'a, T: 'a> {
                t: &'a Range<T>,
                field_idx: u8,
            }

            impl<'a, T> serde::ser::MapVisitor for Visitor<'a, T>
                where T: serde::Serialize,
            {
                fn visit<S>(&mut self, serializer: &mut S) -> Result<Option<()>, S::Error>
                    where S: serde::Serializer,
                {
                    match self.field_idx {
                        0 => {
                            self.field_idx += 1;
                            Ok(Some(try!(serializer.serialize_struct_elt("min", &self.t.min))))
                        },
                        1 => {
                            self.field_idx += 1;
                            Ok(Some(try!(serializer.serialize_struct_elt("max", &self.t.max))))
                        },
                        _ => Ok(None),
                    }
                }

                fn len(&self) -> Option<usize> {
                    Some(2)
                }
            }

            serializer.serialize_struct("Range", Visitor { t: self, field_idx: 0 })
        }
    }

    impl<T> serde::Deserialize for Range<T>
        where T: serde::Deserialize,
    {
        fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error>
            where D: serde::Deserializer,
        {
            struct Visitor<T> {
                t: std::marker::PhantomData<T>,
            };

            impl<T> serde::de::Visitor for Visitor<T>
                where T: serde::Deserialize,
            {
                type Value = Range<T>;

                fn visit_map<V>(&mut self, mut visitor: V) -> Result<Range<T>, V::Error>
                    where V: serde::de::MapVisitor,
                {
                    let mut min = None;
                    let mut max = None;

                    enum Field { Min, Max }

                    impl serde::Deserialize for Field {
                        fn deserialize<D>(deserializer: &mut D) -> Result<Field, D::Error>
                            where D: serde::de::Deserializer,
                        {
                            struct FieldVisitor;

                            impl serde::de::Visitor for FieldVisitor {
                                type Value = Field;

                                fn visit_str<E>(&mut self, value: &str) -> Result<Field, E>
                                    where E: serde::de::Error,
                                {
                                    match value {
                                        "min" => Ok(Field::Min),
                                        "max" => Ok(Field::Max),
                                        _ => Err(serde::de::Error::custom("expected min or max")),
                                    }
                                }
                            }

                            deserializer.deserialize(FieldVisitor)
                        }
                    }

                    loop {
                        match try!(visitor.visit_key()) {
                            Some(Field::Min) => { min = Some(try!(visitor.visit_value())); },
                            Some(Field::Max) => { max = Some(try!(visitor.visit_value())); },
                            None => { break; }
                        }
                    }

                    let min = match min {
                        Some(min) => min,
                        None => return Err(serde::de::Error::missing_field("min")),
                    };

                    let max = match max {
                        Some(max) => max,
                        None => return Err(serde::de::Error::missing_field("max")),
                    };

                    try!(visitor.end());

                    Ok(Range { min: min, max: max })
                }
            }

            static FIELDS: &'static [&'static str] = &["min", "max"];

            let visitor = Visitor { t: std::marker::PhantomData };

            deserializer.deserialize_struct("Range", FIELDS, visitor)
        }
    }

    #[test]
    fn test() {
        extern crate serde_json;

        let range = Range { min: 220.0, max: 440.0 };
        let serialized = serde_json::to_string(&range).unwrap();

        println!("{}", serialized);
        assert_eq!("{\"min\":220,\"max\":440}", serialized);
        
        let deserialized: Range<f32> = serde_json::from_str(&serialized).unwrap();

        println!("{:?}", deserialized);
        assert_eq!(range, deserialized);
    }

}


mod sample {
    use super::serde;
    use map::Sample;
    use std;

    impl<A> serde::Serialize for Sample<A>
        where A: serde::Serialize,
    {
        fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
            where S: serde::Serializer,
        {
            struct Visitor<'a, A: 'a> {
                t: &'a Sample<A>,
                field_idx: u8,
            }

            impl<'a, A> serde::ser::MapVisitor for Visitor<'a, A>
                where A: serde::Serialize,
            {
                fn visit<S>(&mut self, serializer: &mut S) -> Result<Option<()>, S::Error>
                    where S: serde::Serializer,
                {
                    match self.field_idx {
                        0 => {
                            self.field_idx += 1;
                            Ok(Some(try!(serializer.serialize_struct_elt("base_hz", &self.t.base_hz))))
                        },
                        1 => {
                            self.field_idx += 1;
                            Ok(Some(try!(serializer.serialize_struct_elt("base_vel", &self.t.base_vel))))
                        },
                        2 => {
                            self.field_idx += 1;
                            Ok(Some(try!(serializer.serialize_struct_elt("audio", &self.t.audio))))
                        },
                        _ => Ok(None),
                    }
                }

                fn len(&self) -> Option<usize> {
                    Some(3)
                }
            }

            serializer.serialize_struct("Sample", Visitor { t: self, field_idx: 0 })
        }
    }

    impl<A> serde::Deserialize for Sample<A>
        where A: serde::Deserialize,
    {
        fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error>
            where D: serde::Deserializer,
        {
            struct Visitor<A> {
                t: std::marker::PhantomData<A>,
            };

            impl<A> serde::de::Visitor for Visitor<A>
                where A: serde::Deserialize,
            {
                type Value = Sample<A>;

                fn visit_map<V>(&mut self, mut visitor: V) -> Result<Sample<A>, V::Error>
                    where V: serde::de::MapVisitor,
                {
                    let mut base_hz = None;
                    let mut base_vel = None;
                    let mut audio = None;

                    enum Field { BaseHz, BaseVel, Audio }

                    impl serde::Deserialize for Field {
                        fn deserialize<D>(deserializer: &mut D) -> Result<Field, D::Error>
                            where D: serde::de::Deserializer,
                        {
                            struct FieldVisitor;

                            impl serde::de::Visitor for FieldVisitor {
                                type Value = Field;

                                fn visit_str<E>(&mut self, value: &str) -> Result<Field, E>
                                    where E: serde::de::Error,
                                {
                                    match value {
                                        "base_hz" => Ok(Field::BaseHz),
                                        "base_vel" => Ok(Field::BaseVel),
                                        "audio" => Ok(Field::Audio),
                                        _ => Err(serde::de::Error::custom("expected base_hz, base_vel or audio")),
                                    }
                                }
                            }

                            deserializer.deserialize(FieldVisitor)
                        }
                    }

                    loop {
                        match try!(visitor.visit_key()) {
                            Some(Field::BaseHz) => { base_hz = Some(try!(visitor.visit_value())); },
                            Some(Field::BaseVel) => { base_vel = Some(try!(visitor.visit_value())); },
                            Some(Field::Audio) => { audio = Some(try!(visitor.visit_value())); },
                            None => { break; }
                        }
                    }

                    let base_hz = match base_hz {
                        Some(base_hz) => base_hz,
                        None => return Err(serde::de::Error::missing_field("base_hz")),
                    };

                    let base_vel = match base_vel {
                        Some(base_vel) => base_vel,
                        None => return Err(serde::de::Error::missing_field("base_vel")),
                    };

                    let audio = match audio {
                        Some(audio) => audio,
                        None => return Err(serde::de::Error::missing_field("audio")),
                    };

                    try!(visitor.end());

                    Ok(Sample {
                        base_hz: base_hz,
                        base_vel: base_vel,
                        audio: audio,
                    })
                }
            }

            static FIELDS: &'static [&'static str] = &["base_hz", "base_vel", "audio"];

            let visitor = Visitor { t: std::marker::PhantomData };

            deserializer.deserialize_struct("Sample", FIELDS, visitor)
        }
    }

    #[test]
    fn test() {
        extern crate serde_json;

        use map;

        impl map::Audio for () {
            type Frame = [f32; 2];
            fn data(&self) -> &[Self::Frame] { &[] }
        }

        let sample = Sample { base_hz: 440.0.into(), base_vel: 1.0, audio: () };
        let serialized = serde_json::to_string(&sample).unwrap();

        println!("{}", serialized);
        assert_eq!("{\"base_hz\":440,\"base_vel\":1,\"audio\":null}", serialized);
        
        let deserialized: Sample<()> = serde_json::from_str(&serialized).unwrap();

        println!("{:?}", deserialized);
        assert_eq!(sample, deserialized);
    }

}

mod sample_over_range {
    use super::serde;
    use map::SampleOverRange;
    use std;

    impl<A> serde::Serialize for SampleOverRange<A>
        where A: serde::Serialize,
    {
        fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
            where S: serde::Serializer,
        {
            struct Visitor<'a, A: 'a> {
                t: &'a SampleOverRange<A>,
                field_idx: u8,
            }

            impl<'a, A> serde::ser::MapVisitor for Visitor<'a, A>
                where A: serde::Serialize,
            {
                fn visit<S>(&mut self, serializer: &mut S) -> Result<Option<()>, S::Error>
                    where S: serde::Serializer,
                {
                    match self.field_idx {
                        0 => {
                            self.field_idx += 1;
                            Ok(Some(try!(serializer.serialize_struct_elt("range", &self.t.range))))
                        },
                        1 => {
                            self.field_idx += 1;
                            Ok(Some(try!(serializer.serialize_struct_elt("sample", &self.t.sample))))
                        },
                        _ => Ok(None),
                    }
                }

                fn len(&self) -> Option<usize> {
                    Some(2)
                }
            }

            serializer.serialize_struct("SampleOverRange", Visitor { t: self, field_idx: 0 })
        }
    }

    impl<A> serde::Deserialize for SampleOverRange<A>
        where A: serde::Deserialize,
    {
        fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error>
            where D: serde::Deserializer,
        {
            struct Visitor<A> {
                t: std::marker::PhantomData<A>,
            };

            impl<A> serde::de::Visitor for Visitor<A>
                where A: serde::Deserialize,
            {
                type Value = SampleOverRange<A>;

                fn visit_map<V>(&mut self, mut visitor: V) -> Result<SampleOverRange<A>, V::Error>
                    where V: serde::de::MapVisitor,
                {
                    let mut range = None;
                    let mut sample = None;

                    enum Field { Range, Sample }

                    impl serde::Deserialize for Field {
                        fn deserialize<D>(deserializer: &mut D) -> Result<Field, D::Error>
                            where D: serde::de::Deserializer,
                        {
                            struct FieldVisitor;

                            impl serde::de::Visitor for FieldVisitor {
                                type Value = Field;

                                fn visit_str<E>(&mut self, value: &str) -> Result<Field, E>
                                    where E: serde::de::Error,
                                {
                                    match value {
                                        "range" => Ok(Field::Range),
                                        "sample" => Ok(Field::Sample),
                                        _ => Err(serde::de::Error::custom("expected range or sample")),
                                    }
                                }
                            }

                            deserializer.deserialize(FieldVisitor)
                        }
                    }

                    loop {
                        match try!(visitor.visit_key()) {
                            Some(Field::Range) => { range = Some(try!(visitor.visit_value())); },
                            Some(Field::Sample) => { sample = Some(try!(visitor.visit_value())); },
                            None => { break; }
                        }
                    }

                    let range = match range {
                        Some(range) => range,
                        None => return Err(serde::de::Error::missing_field("range")),
                    };

                    let sample = match sample {
                        Some(sample) => sample,
                        None => return Err(serde::de::Error::missing_field("sample")),
                    };

                    try!(visitor.end());

                    Ok(SampleOverRange { range: range, sample: sample })
                }
            }

            static FIELDS: &'static [&'static str] = &["range", "sample"];

            let visitor = Visitor { t: std::marker::PhantomData };

            deserializer.deserialize_struct("Range", FIELDS, visitor)
        }
    }

    #[test]
    fn test() {
        extern crate serde_json;

        use map;

        // impl map::Audio for () {
        //     type Frame = [f32; 2];
        //     fn data(&self) -> &[Self::Frame] { &[] }
        // }

        let sample = map::Sample { base_hz: 440.0.into(), base_vel: 1.0, audio: () };
        let range = map::HzVelRange {
            hz: map::Range { min: 220.0.into(), max: 440.0.into() },
            vel: map::Range { min: 0.0, max: 1.0 },
        };

        let sample_over_range = SampleOverRange { range: range, sample: sample };
        let serialized = serde_json::to_string(&sample_over_range).unwrap();

        println!("{}", serialized);
        assert_eq!("{\"range\":{\"hz\":{\"min\":220,\"max\":440},\"vel\":{\"min\":0,\"max\":1}},\"sample\":{\"base_hz\":440,\"base_vel\":1,\"audio\":null}}", serialized);
        
        let deserialized: SampleOverRange<()> = serde_json::from_str(&serialized).unwrap();

        println!("{:?}", deserialized);
        assert_eq!(sample_over_range, deserialized);
    }

}


mod hz_vel_range {
    use super::serde;
    use map::HzVelRange;

    impl serde::Serialize for HzVelRange {
        fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
            where S: serde::Serializer,
        {
            struct Visitor<'a> {
                t: &'a HzVelRange,
                field_idx: u8,
            }

            impl<'a> serde::ser::MapVisitor for Visitor<'a> {
                fn visit<S>(&mut self, serializer: &mut S) -> Result<Option<()>, S::Error>
                    where S: serde::Serializer,
                {
                    match self.field_idx {
                        0 => {
                            self.field_idx += 1;
                            Ok(Some(try!(serializer.serialize_struct_elt("hz", &self.t.hz))))
                        },
                        1 => {
                            self.field_idx += 1;
                            Ok(Some(try!(serializer.serialize_struct_elt("vel", &self.t.vel))))
                        },
                        _ => Ok(None),
                    }
                }

                fn len(&self) -> Option<usize> {
                    Some(2)
                }
            }

            serializer.serialize_struct("HzVelRange", Visitor { t: self, field_idx: 0 })
        }
    }

    impl serde::Deserialize for HzVelRange {
        fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error>
            where D: serde::Deserializer,
        {
            struct Visitor;

            impl serde::de::Visitor for Visitor {
                type Value = HzVelRange;

                fn visit_map<V>(&mut self, mut visitor: V) -> Result<HzVelRange, V::Error>
                    where V: serde::de::MapVisitor,
                {
                    let mut hz = None;
                    let mut vel = None;

                    enum Field { Hz, Vel }

                    impl serde::Deserialize for Field {
                        fn deserialize<D>(deserializer: &mut D) -> Result<Field, D::Error>
                            where D: serde::de::Deserializer,
                        {
                            struct FieldVisitor;

                            impl serde::de::Visitor for FieldVisitor {
                                type Value = Field;

                                fn visit_str<E>(&mut self, value: &str) -> Result<Field, E>
                                    where E: serde::de::Error,
                                {
                                    match value {
                                        "hz" => Ok(Field::Hz),
                                        "vel" => Ok(Field::Vel),
                                        _ => Err(serde::de::Error::custom("expected hz or vel")),
                                    }
                                }
                            }

                            deserializer.deserialize(FieldVisitor)
                        }
                    }

                    loop {
                        match try!(visitor.visit_key()) {
                            Some(Field::Hz) => { hz = Some(try!(visitor.visit_value())); },
                            Some(Field::Vel) => { vel = Some(try!(visitor.visit_value())); },
                            None => { break; }
                        }
                    }

                    let hz = match hz {
                        Some(hz) => hz,
                        None => return Err(serde::de::Error::missing_field("hz")),
                    };

                    let vel = match vel {
                        Some(vel) => vel,
                        None => return Err(serde::de::Error::missing_field("vel")),
                    };

                    try!(visitor.end());

                    Ok(HzVelRange { hz: hz, vel: vel })
                }
            }

            static FIELDS: &'static [&'static str] = &["hz", "vel"];

            let visitor = Visitor;

            deserializer.deserialize_struct("HzVelRange", FIELDS, visitor)
        }
    }

    #[test]
    fn test() {
        extern crate serde_json;
        use map;

        let range = HzVelRange {
            hz: map::Range { min: 220.0.into(), max: 440.0.into() },
            vel: map::Range { min: 0.0, max: 1.0 },
        };
        let serialized = serde_json::to_string(&range).unwrap();

        println!("{}", serialized);
        assert_eq!("{\"hz\":{\"min\":220,\"max\":440},\"vel\":{\"min\":0,\"max\":1}}", serialized);
        
        let deserialized: HzVelRange = serde_json::from_str(&serialized).unwrap();

        println!("{:?}", deserialized);
        assert_eq!(range, deserialized);
    }

}


mod map {
    use super::serde;
    use map::Map;
    use std;

    impl<A> serde::Serialize for Map<A>
        where A: serde::Serialize,
    {
        fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
            where S: serde::Serializer,
        {
            struct Visitor<'a, A: 'a> {
                t: &'a Map<A>,
                field_idx: u8,
            }

            impl<'a, A> serde::ser::MapVisitor for Visitor<'a, A>
                where A: serde::Serialize,
            {
                fn visit<S>(&mut self, serializer: &mut S) -> Result<Option<()>, S::Error>
                    where S: serde::Serializer,
                {
                    match self.field_idx {
                        0 => {
                            self.field_idx += 1;
                            Ok(Some(try!(serializer.serialize_struct_elt("pairs", &self.t.pairs))))
                        },
                        _ => Ok(None),
                    }
                }

                fn len(&self) -> Option<usize> {
                    Some(1)
                }
            }

            serializer.serialize_struct("Map", Visitor { t: self, field_idx: 0 })
        }
    }

    impl<A> serde::Deserialize for Map<A>
        where A: serde::Deserialize,
    {
        fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error>
            where D: serde::Deserializer,
        {
            struct Visitor<A> {
                t: std::marker::PhantomData<A>,
            };

            impl<A> serde::de::Visitor for Visitor<A>
                where A: serde::Deserialize,
            {
                type Value = Map<A>;

                fn visit_map<V>(&mut self, mut visitor: V) -> Result<Map<A>, V::Error>
                    where V: serde::de::MapVisitor,
                {
                    let mut pairs = None;

                    enum Field { Pairs }

                    impl serde::Deserialize for Field {
                        fn deserialize<D>(deserializer: &mut D) -> Result<Field, D::Error>
                            where D: serde::de::Deserializer,
                        {
                            struct FieldVisitor;

                            impl serde::de::Visitor for FieldVisitor {
                                type Value = Field;

                                fn visit_str<E>(&mut self, value: &str) -> Result<Field, E>
                                    where E: serde::de::Error,
                                {
                                    match value {
                                        "pairs" => Ok(Field::Pairs),
                                        _ => Err(serde::de::Error::custom("expected pairs")),
                                    }
                                }
                            }

                            deserializer.deserialize(FieldVisitor)
                        }
                    }

                    loop {
                        match try!(visitor.visit_key()) {
                            Some(Field::Pairs) => { pairs = Some(try!(visitor.visit_value())); },
                            None => { break; }
                        }
                    }

                    let pairs = match pairs {
                        Some(pairs) => pairs,
                        None => return Err(serde::de::Error::missing_field("pairs")),
                    };

                    try!(visitor.end());

                    Ok(Map { pairs: pairs })
                }
            }

            static FIELDS: &'static [&'static str] = &["pairs"];

            let visitor = Visitor { t: std::marker::PhantomData };

            deserializer.deserialize_struct("Map", FIELDS, visitor)
        }
    }

    #[test]
    fn test() {
        extern crate serde_json;

        let map: Map<()> = Map::empty();
        let serialized = serde_json::to_string(&map).unwrap();

        println!("{}", serialized);
        assert_eq!("{\"pairs\":[]}", serialized);
        
        let deserialized: Map<()> = serde_json::from_str(&serialized).unwrap();

        println!("{:?}", deserialized);
        assert_eq!(map, deserialized);
    }

}


mod sampler {
    use instrument;
    use map;
    use super::serde;
    use sampler::{self, Sampler};
    use std;

    impl<M, NFG, A> serde::Serialize for Sampler<M, NFG, A>
        where M: serde::Serialize,
              NFG: serde::Serialize + instrument::NoteFreqGenerator,
              NFG::NoteFreq: serde::Serialize,
              A: serde::Serialize + map::Audio,
    {
        fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
            where S: serde::Serializer,
        {
            struct Visitor<'a, M: 'a, NFG: 'a, A: 'a>
                where NFG: instrument::NoteFreqGenerator,
                      A: map::Audio,
            {
                t: &'a Sampler<M, NFG, A>,
                field_idx: u8,
            }

            impl<'a, M, NFG, A> serde::ser::MapVisitor for Visitor<'a, M, NFG, A>
                where M: serde::Serialize,
                      NFG: serde::Serialize + instrument::NoteFreqGenerator,
                      NFG::NoteFreq: serde::Serialize,
                      A: serde::Serialize + map::Audio,
            {
                fn visit<S>(&mut self, serializer: &mut S) -> Result<Option<()>, S::Error>
                    where S: serde::Serializer,
                {
                    match self.field_idx {
                        0 => {
                            self.field_idx += 1;
                            Ok(Some(try!(serializer.serialize_struct_elt("instrument", &self.t.instrument))))
                        },
                        1 => {
                            self.field_idx += 1;
                            Ok(Some(try!(serializer.serialize_struct_elt("map", &self.t.map))))
                        },
                        2 => {
                            self.field_idx += 1;
                            let num_voices = self.t.voice_count();
                            Ok(Some(try!(serializer.serialize_struct_elt("voices", num_voices))))
                        },
                        _ => Ok(None),
                    }
                }

                fn len(&self) -> Option<usize> {
                    Some(3)
                }
            }

            serializer.serialize_struct("Sampler", Visitor { t: self, field_idx: 0 })
        }
    }

    impl<M, NFG, A> serde::Deserialize for Sampler<M, NFG, A>
        where M: serde::Deserialize,
              NFG: serde::Deserialize + instrument::NoteFreqGenerator,
              NFG::NoteFreq: serde::Deserialize,
              A: serde::Deserialize + map::Audio,
    {
        fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error>
            where D: serde::Deserializer,
        {
            struct Visitor<M, NFG, A> {
                m: std::marker::PhantomData<M>,
                nfg: std::marker::PhantomData<NFG>,
                a: std::marker::PhantomData<A>,
            };

            impl<M, NFG, A> serde::de::Visitor for Visitor<M, NFG, A>
                where M: serde::Deserialize,
                      NFG: serde::Deserialize + instrument::NoteFreqGenerator,
                      NFG::NoteFreq: serde::Deserialize,
                      A: serde::Deserialize + map::Audio,
            {
                type Value = Sampler<M, NFG, A>;

                fn visit_map<V>(&mut self, mut visitor: V) -> Result<Sampler<M, NFG, A>, V::Error>
                    where V: serde::de::MapVisitor,
                {
                    let mut instrument = None;
                    let mut map = None;
                    let mut num_voices = None;

                    enum Field { Instrument, Map, Voices }

                    impl serde::Deserialize for Field {
                        fn deserialize<D>(deserializer: &mut D) -> Result<Field, D::Error>
                            where D: serde::de::Deserializer,
                        {
                            struct FieldVisitor;

                            impl serde::de::Visitor for FieldVisitor {
                                type Value = Field;

                                fn visit_str<E>(&mut self, value: &str) -> Result<Field, E>
                                    where E: serde::de::Error,
                                {
                                    match value {
                                        "instrument" => Ok(Field::Instrument),
                                        "map" => Ok(Field::Map),
                                        "voices" => Ok(Field::Voices),
                                        _ => Err(serde::de::Error::custom("expected instrument, map or voices")),
                                    }
                                }
                            }

                            deserializer.deserialize(FieldVisitor)
                        }
                    }

                    loop {
                        match try!(visitor.visit_key()) {
                            Some(Field::Instrument) => { instrument = Some(try!(visitor.visit_value())); },
                            Some(Field::Map) => { map = Some(try!(visitor.visit_value())); },
                            Some(Field::Voices) => { num_voices = Some(try!(visitor.visit_value())); },
                            None => { break; }
                        }
                    }

                    let instrument = match instrument {
                        Some(instrument) => instrument,
                        None => return Err(serde::de::Error::missing_field("instrument")),
                    };

                    let map = match map {
                        Some(map) => map,
                        None => return Err(serde::de::Error::missing_field("map")),
                    };

                    let num_voices = match num_voices {
                        Some(num_voices) => num_voices,
                        None => return Err(serde::de::Error::missing_field("voices")),
                    };

                    try!(visitor.end());

                    Ok(sampler::private::new(instrument, map, num_voices))
                }
            }

            static FIELDS: &'static [&'static str] = &["instrument", "map", "voices"];

            let visitor = Visitor {
                m: std::marker::PhantomData,
                nfg: std::marker::PhantomData,
                a: std::marker::PhantomData,
            };

            deserializer.deserialize_struct("Sampler", FIELDS, visitor)
        }
    }

    #[test]
    fn test() {
        extern crate serde_json;
        use instrument;
        use map;

        let map: map::Map<()> = map::Map::empty();
        let sampler = Sampler::legato((), map);
        let serialized = serde_json::to_string(&sampler).unwrap();

        println!("{}", serialized);
        
        let deserialized: Sampler<instrument::mode::Mono, (), ()> =
            serde_json::from_str(&serialized).unwrap();

        println!("{:?}", deserialized);
        assert_eq!(&sampler.instrument, &deserialized.instrument);
        assert_eq!(&sampler.map, &deserialized.map);
        assert_eq!(sampler.voice_count(), deserialized.voice_count());
    }

}


#[cfg(feature="wav")]
mod wav_audio {
    extern crate find_folder;

    use map::wav;
    use sample;
    use super::serde;
    use std;

    impl<F> serde::Serialize for wav::Audio<F> {
        fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
            where S: serde::Serializer,
        {
            struct Visitor<'a, F: 'a> {
                t: &'a wav::Audio<F>,
                field_idx: u8,
            }

            impl<'a, F> serde::ser::MapVisitor for Visitor<'a, F> {
                fn visit<S>(&mut self, serializer: &mut S) -> Result<Option<()>, S::Error>
                    where S: serde::Serializer,
                {
                    match self.field_idx {
                        0 => {
                            self.field_idx += 1;
                            Ok(Some(try!(serializer.serialize_struct_elt("path", &self.t.path))))
                        },
                        1 => {
                            self.field_idx += 1;
                            Ok(Some(try!(serializer.serialize_struct_elt("sample_hz", &self.t.sample_hz))))
                        },
                        _ => Ok(None),
                    }
                }

                fn len(&self) -> Option<usize> {
                    Some(2)
                }
            }

            serializer.serialize_struct("Audio", Visitor { t: self, field_idx: 0 })
        }
    }

    impl<F> serde::Deserialize for wav::Audio<F>
        where F: sample::Frame + serde::Deserialize,
              F::Sample: sample::Duplex<f64> + sample::Duplex<i32>,
              Box<[F::Sample]>: sample::ToBoxedFrameSlice<F>,
    {
        fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error>
            where D: serde::Deserializer,
        {
            struct Visitor<F> {
                f: std::marker::PhantomData<F>,
            };

            impl<F> serde::de::Visitor for Visitor<F>
                where F: sample::Frame + serde::Deserialize,
                      F::Sample: sample::Duplex<f64> + sample::Duplex<i32>,
                      Box<[F::Sample]>: sample::ToBoxedFrameSlice<F>,
            {
                type Value = wav::Audio<F>;

                fn visit_map<V>(&mut self, mut visitor: V) -> Result<wav::Audio<F>, V::Error>
                    where V: serde::de::MapVisitor,
                {
                    let mut path = None;
                    let mut sample_hz = None;

                    enum Field { Path, SampleHz }

                    impl serde::Deserialize for Field {
                        fn deserialize<D>(deserializer: &mut D) -> Result<Field, D::Error>
                            where D: serde::de::Deserializer,
                        {
                            struct FieldVisitor;

                            impl serde::de::Visitor for FieldVisitor {
                                type Value = Field;

                                fn visit_str<E>(&mut self, value: &str) -> Result<Field, E>
                                    where E: serde::de::Error,
                                {
                                    match value {
                                        "path" => Ok(Field::Path),
                                        "sample_hz" => Ok(Field::SampleHz),
                                        _ => Err(serde::de::Error::custom("expected path or sample_hz")),
                                    }
                                }
                            }

                            deserializer.deserialize(FieldVisitor)
                        }
                    }

                    loop {
                        match try!(visitor.visit_key()) {
                            Some(Field::Path) => { path = Some(try!(visitor.visit_value())); },
                            Some(Field::SampleHz) => { sample_hz = Some(try!(visitor.visit_value())); },
                            None => { break; }
                        }
                    }

                    let path: std::path::PathBuf = match path {
                        Some(path) => path,
                        None => return Err(serde::de::Error::missing_field("path")),
                    };

                    let sample_hz = match sample_hz {
                        Some(sample_hz) => sample_hz,
                        None => return Err(serde::de::Error::missing_field("sample_hz")),
                    };

                    try!(visitor.end());

                    wav::Audio::from_file(path, sample_hz).map_err(|e| {
                        serde::de::Error::custom(std::error::Error::description(&e))
                    })
                }
            }

            static FIELDS: &'static [&'static str] = &["path", "sample_hz"];

            let visitor = Visitor { f: std::marker::PhantomData };

            deserializer.deserialize_struct("Audio", FIELDS, visitor)
        }
    }

    #[test]
    fn test() {
        extern crate serde_json;

        const THUMB_PIANO: &'static str = "thumbpiano A#3.wav";
        const SAMPLE_HZ: f64 = 44_100.0;

        let assets = find_folder::Search::ParentsThenKids(5, 5).for_folder("assets").unwrap();
        let path = assets.join(THUMB_PIANO);
        let audio = wav::Audio::<[i16; 2]>::from_file(path, SAMPLE_HZ).unwrap();

        let serialized = serde_json::to_string(&audio).unwrap();

        println!("{}", serialized);
        assert_eq!("{\"path\":\"/Users/Mitch/Programming/Rust/sampler/assets/thumbpiano A#3.wav\",\"sample_hz\":44100}", serialized);
        
        let deserialized: wav::Audio<[i16; 2]> = serde_json::from_str(&serialized).unwrap();

        assert_eq!(audio, deserialized);
    }
}
