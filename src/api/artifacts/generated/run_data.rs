// This file is generated by rust-protobuf 2.8.2. Do not edit
// @generated

// https://github.com/Manishearth/rust-clippy/issues/702
#![allow(unknown_lints)]
#![allow(clippy::all)]

#![cfg_attr(rustfmt, rustfmt_skip)]

#![allow(box_pointers)]
#![allow(dead_code)]
#![allow(missing_docs)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(trivial_casts)]
#![allow(unsafe_code)]
#![allow(unused_imports)]
#![allow(unused_results)]
//! Generated file from `src/api/artifacts/run_data.proto`

use protobuf::Message as Message_imported_for_functions;
use protobuf::ProtobufEnum as ProtobufEnum_imported_for_functions;

/// Generated files are compatible only with the same version
/// of protobuf runtime.
const _PROTOBUF_VERSION_CHECK: () = ::protobuf::VERSION_2_8_2;

#[derive(PartialEq,Clone,Default)]
pub struct RunData {
    // message fields
    pub client_creation_time: ::protobuf::SingularPtrField<::protobuf::well_known_types::Timestamp>,
    pub root_group_id: ::protobuf::SingularPtrField<super::artifact::ArtifactId>,
    // special fields
    pub unknown_fields: ::protobuf::UnknownFields,
    pub cached_size: ::protobuf::CachedSize,
}

impl<'a> ::std::default::Default for &'a RunData {
    fn default() -> &'a RunData {
        <RunData as ::protobuf::Message>::default_instance()
    }
}

impl RunData {
    pub fn new() -> RunData {
        ::std::default::Default::default()
    }

    // .google.protobuf.Timestamp client_creation_time = 3;


    pub fn get_client_creation_time(&self) -> &::protobuf::well_known_types::Timestamp {
        self.client_creation_time.as_ref().unwrap_or_else(|| ::protobuf::well_known_types::Timestamp::default_instance())
    }
    pub fn clear_client_creation_time(&mut self) {
        self.client_creation_time.clear();
    }

    pub fn has_client_creation_time(&self) -> bool {
        self.client_creation_time.is_some()
    }

    // Param is passed by value, moved
    pub fn set_client_creation_time(&mut self, v: ::protobuf::well_known_types::Timestamp) {
        self.client_creation_time = ::protobuf::SingularPtrField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_client_creation_time(&mut self) -> &mut ::protobuf::well_known_types::Timestamp {
        if self.client_creation_time.is_none() {
            self.client_creation_time.set_default();
        }
        self.client_creation_time.as_mut().unwrap()
    }

    // Take field
    pub fn take_client_creation_time(&mut self) -> ::protobuf::well_known_types::Timestamp {
        self.client_creation_time.take().unwrap_or_else(|| ::protobuf::well_known_types::Timestamp::new())
    }

    // .observation_tools.proto.ArtifactId root_group_id = 5;


    pub fn get_root_group_id(&self) -> &super::artifact::ArtifactId {
        self.root_group_id.as_ref().unwrap_or_else(|| super::artifact::ArtifactId::default_instance())
    }
    pub fn clear_root_group_id(&mut self) {
        self.root_group_id.clear();
    }

    pub fn has_root_group_id(&self) -> bool {
        self.root_group_id.is_some()
    }

    // Param is passed by value, moved
    pub fn set_root_group_id(&mut self, v: super::artifact::ArtifactId) {
        self.root_group_id = ::protobuf::SingularPtrField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_root_group_id(&mut self) -> &mut super::artifact::ArtifactId {
        if self.root_group_id.is_none() {
            self.root_group_id.set_default();
        }
        self.root_group_id.as_mut().unwrap()
    }

    // Take field
    pub fn take_root_group_id(&mut self) -> super::artifact::ArtifactId {
        self.root_group_id.take().unwrap_or_else(|| super::artifact::ArtifactId::new())
    }
}

impl ::protobuf::Message for RunData {
    fn is_initialized(&self) -> bool {
        for v in &self.client_creation_time {
            if !v.is_initialized() {
                return false;
            }
        };
        for v in &self.root_group_id {
            if !v.is_initialized() {
                return false;
            }
        };
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream<'_>) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                3 => {
                    ::protobuf::rt::read_singular_message_into(wire_type, is, &mut self.client_creation_time)?;
                },
                5 => {
                    ::protobuf::rt::read_singular_message_into(wire_type, is, &mut self.root_group_id)?;
                },
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        if let Some(ref v) = self.client_creation_time.as_ref() {
            let len = v.compute_size();
            my_size += 1 + ::protobuf::rt::compute_raw_varint32_size(len) + len;
        }
        if let Some(ref v) = self.root_group_id.as_ref() {
            let len = v.compute_size();
            my_size += 1 + ::protobuf::rt::compute_raw_varint32_size(len) + len;
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream<'_>) -> ::protobuf::ProtobufResult<()> {
        if let Some(ref v) = self.client_creation_time.as_ref() {
            os.write_tag(3, ::protobuf::wire_format::WireTypeLengthDelimited)?;
            os.write_raw_varint32(v.get_cached_size())?;
            v.write_to_with_cached_sizes(os)?;
        }
        if let Some(ref v) = self.root_group_id.as_ref() {
            os.write_tag(5, ::protobuf::wire_format::WireTypeLengthDelimited)?;
            os.write_raw_varint32(v.get_cached_size())?;
            v.write_to_with_cached_sizes(os)?;
        }
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &dyn (::std::any::Any) {
        self as &dyn (::std::any::Any)
    }
    fn as_any_mut(&mut self) -> &mut dyn (::std::any::Any) {
        self as &mut dyn (::std::any::Any)
    }
    fn into_any(self: Box<Self>) -> ::std::boxed::Box<dyn (::std::any::Any)> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        Self::descriptor_static()
    }

    fn new() -> RunData {
        RunData::new()
    }

    fn descriptor_static() -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_singular_ptr_field_accessor::<_, ::protobuf::types::ProtobufTypeMessage<::protobuf::well_known_types::Timestamp>>(
                    "client_creation_time",
                    |m: &RunData| { &m.client_creation_time },
                    |m: &mut RunData| { &mut m.client_creation_time },
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_ptr_field_accessor::<_, ::protobuf::types::ProtobufTypeMessage<super::artifact::ArtifactId>>(
                    "root_group_id",
                    |m: &RunData| { &m.root_group_id },
                    |m: &mut RunData| { &mut m.root_group_id },
                ));
                ::protobuf::reflect::MessageDescriptor::new::<RunData>(
                    "RunData",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }

    fn default_instance() -> &'static RunData {
        static mut instance: ::protobuf::lazy::Lazy<RunData> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const RunData,
        };
        unsafe {
            instance.get(RunData::new)
        }
    }
}

impl ::protobuf::Clear for RunData {
    fn clear(&mut self) {
        self.client_creation_time.clear();
        self.root_group_id.clear();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for RunData {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for RunData {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

static file_descriptor_proto_data: &'static [u8] = b"\
    \n\x20src/api/artifacts/run_data.proto\x12\x17observation_tools.proto\
    \x1a\x20src/api/artifacts/artifact.proto\x1a\x1fgoogle/protobuf/timestam\
    p.proto\"\xac\x01\n\x07RunData\x12L\n\x14client_creation_time\x18\x03\
    \x20\x01(\x0b2\x1a.google.protobuf.TimestampR\x12clientCreationTime\x12G\
    \n\rroot_group_id\x18\x05\x20\x01(\x0b2#.observation_tools.proto.Artifac\
    tIdR\x0brootGroupIdJ\x04\x08\x01\x10\x02J\x04\x08\x02\x10\x03B\x1b\n\x17\
    observation_tools.protoP\x01b\x06proto3\
";

static mut file_descriptor_proto_lazy: ::protobuf::lazy::Lazy<::protobuf::descriptor::FileDescriptorProto> = ::protobuf::lazy::Lazy {
    lock: ::protobuf::lazy::ONCE_INIT,
    ptr: 0 as *const ::protobuf::descriptor::FileDescriptorProto,
};

fn parse_descriptor_proto() -> ::protobuf::descriptor::FileDescriptorProto {
    ::protobuf::parse_from_bytes(file_descriptor_proto_data).unwrap()
}

pub fn file_descriptor_proto() -> &'static ::protobuf::descriptor::FileDescriptorProto {
    unsafe {
        file_descriptor_proto_lazy.get(|| {
            parse_descriptor_proto()
        })
    }
}
