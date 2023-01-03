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
//! Generated file from `src/api/artifacts/api_context.proto`

use protobuf::Message as Message_imported_for_functions;
use protobuf::ProtobufEnum as ProtobufEnum_imported_for_functions;

/// Generated files are compatible only with the same version
/// of protobuf runtime.
const _PROTOBUF_VERSION_CHECK: () = ::protobuf::VERSION_2_8_2;

#[derive(PartialEq,Clone,Default)]
pub struct ArtifactGroupUploaderData {
    // message fields
    pub project_id: ::std::string::String,
    pub run_id: ::protobuf::SingularPtrField<super::run_id::RunId>,
    pub id: ::protobuf::SingularPtrField<super::artifact::ArtifactId>,
    pub ancestor_group_ids: ::protobuf::RepeatedField<super::artifact::ArtifactId>,
    // special fields
    pub unknown_fields: ::protobuf::UnknownFields,
    pub cached_size: ::protobuf::CachedSize,
}

impl<'a> ::std::default::Default for &'a ArtifactGroupUploaderData {
    fn default() -> &'a ArtifactGroupUploaderData {
        <ArtifactGroupUploaderData as ::protobuf::Message>::default_instance()
    }
}

impl ArtifactGroupUploaderData {
    pub fn new() -> ArtifactGroupUploaderData {
        ::std::default::Default::default()
    }

    // string project_id = 4;


    pub fn get_project_id(&self) -> &str {
        &self.project_id
    }
    pub fn clear_project_id(&mut self) {
        self.project_id.clear();
    }

    // Param is passed by value, moved
    pub fn set_project_id(&mut self, v: ::std::string::String) {
        self.project_id = v;
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_project_id(&mut self) -> &mut ::std::string::String {
        &mut self.project_id
    }

    // Take field
    pub fn take_project_id(&mut self) -> ::std::string::String {
        ::std::mem::replace(&mut self.project_id, ::std::string::String::new())
    }

    // .observation_tools.proto.RunId run_id = 5;


    pub fn get_run_id(&self) -> &super::run_id::RunId {
        self.run_id.as_ref().unwrap_or_else(|| super::run_id::RunId::default_instance())
    }
    pub fn clear_run_id(&mut self) {
        self.run_id.clear();
    }

    pub fn has_run_id(&self) -> bool {
        self.run_id.is_some()
    }

    // Param is passed by value, moved
    pub fn set_run_id(&mut self, v: super::run_id::RunId) {
        self.run_id = ::protobuf::SingularPtrField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_run_id(&mut self) -> &mut super::run_id::RunId {
        if self.run_id.is_none() {
            self.run_id.set_default();
        }
        self.run_id.as_mut().unwrap()
    }

    // Take field
    pub fn take_run_id(&mut self) -> super::run_id::RunId {
        self.run_id.take().unwrap_or_else(|| super::run_id::RunId::new())
    }

    // .observation_tools.proto.ArtifactId id = 3;


    pub fn get_id(&self) -> &super::artifact::ArtifactId {
        self.id.as_ref().unwrap_or_else(|| super::artifact::ArtifactId::default_instance())
    }
    pub fn clear_id(&mut self) {
        self.id.clear();
    }

    pub fn has_id(&self) -> bool {
        self.id.is_some()
    }

    // Param is passed by value, moved
    pub fn set_id(&mut self, v: super::artifact::ArtifactId) {
        self.id = ::protobuf::SingularPtrField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_id(&mut self) -> &mut super::artifact::ArtifactId {
        if self.id.is_none() {
            self.id.set_default();
        }
        self.id.as_mut().unwrap()
    }

    // Take field
    pub fn take_id(&mut self) -> super::artifact::ArtifactId {
        self.id.take().unwrap_or_else(|| super::artifact::ArtifactId::new())
    }

    // repeated .observation_tools.proto.ArtifactId ancestor_group_ids = 6;


    pub fn get_ancestor_group_ids(&self) -> &[super::artifact::ArtifactId] {
        &self.ancestor_group_ids
    }
    pub fn clear_ancestor_group_ids(&mut self) {
        self.ancestor_group_ids.clear();
    }

    // Param is passed by value, moved
    pub fn set_ancestor_group_ids(&mut self, v: ::protobuf::RepeatedField<super::artifact::ArtifactId>) {
        self.ancestor_group_ids = v;
    }

    // Mutable pointer to the field.
    pub fn mut_ancestor_group_ids(&mut self) -> &mut ::protobuf::RepeatedField<super::artifact::ArtifactId> {
        &mut self.ancestor_group_ids
    }

    // Take field
    pub fn take_ancestor_group_ids(&mut self) -> ::protobuf::RepeatedField<super::artifact::ArtifactId> {
        ::std::mem::replace(&mut self.ancestor_group_ids, ::protobuf::RepeatedField::new())
    }
}

impl ::protobuf::Message for ArtifactGroupUploaderData {
    fn is_initialized(&self) -> bool {
        for v in &self.run_id {
            if !v.is_initialized() {
                return false;
            }
        };
        for v in &self.id {
            if !v.is_initialized() {
                return false;
            }
        };
        for v in &self.ancestor_group_ids {
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
                4 => {
                    ::protobuf::rt::read_singular_proto3_string_into(wire_type, is, &mut self.project_id)?;
                },
                5 => {
                    ::protobuf::rt::read_singular_message_into(wire_type, is, &mut self.run_id)?;
                },
                3 => {
                    ::protobuf::rt::read_singular_message_into(wire_type, is, &mut self.id)?;
                },
                6 => {
                    ::protobuf::rt::read_repeated_message_into(wire_type, is, &mut self.ancestor_group_ids)?;
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
        if !self.project_id.is_empty() {
            my_size += ::protobuf::rt::string_size(4, &self.project_id);
        }
        if let Some(ref v) = self.run_id.as_ref() {
            let len = v.compute_size();
            my_size += 1 + ::protobuf::rt::compute_raw_varint32_size(len) + len;
        }
        if let Some(ref v) = self.id.as_ref() {
            let len = v.compute_size();
            my_size += 1 + ::protobuf::rt::compute_raw_varint32_size(len) + len;
        }
        for value in &self.ancestor_group_ids {
            let len = value.compute_size();
            my_size += 1 + ::protobuf::rt::compute_raw_varint32_size(len) + len;
        };
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream<'_>) -> ::protobuf::ProtobufResult<()> {
        if !self.project_id.is_empty() {
            os.write_string(4, &self.project_id)?;
        }
        if let Some(ref v) = self.run_id.as_ref() {
            os.write_tag(5, ::protobuf::wire_format::WireTypeLengthDelimited)?;
            os.write_raw_varint32(v.get_cached_size())?;
            v.write_to_with_cached_sizes(os)?;
        }
        if let Some(ref v) = self.id.as_ref() {
            os.write_tag(3, ::protobuf::wire_format::WireTypeLengthDelimited)?;
            os.write_raw_varint32(v.get_cached_size())?;
            v.write_to_with_cached_sizes(os)?;
        }
        for v in &self.ancestor_group_ids {
            os.write_tag(6, ::protobuf::wire_format::WireTypeLengthDelimited)?;
            os.write_raw_varint32(v.get_cached_size())?;
            v.write_to_with_cached_sizes(os)?;
        };
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

    fn new() -> ArtifactGroupUploaderData {
        ArtifactGroupUploaderData::new()
    }

    fn descriptor_static() -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                    "project_id",
                    |m: &ArtifactGroupUploaderData| { &m.project_id },
                    |m: &mut ArtifactGroupUploaderData| { &mut m.project_id },
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_ptr_field_accessor::<_, ::protobuf::types::ProtobufTypeMessage<super::run_id::RunId>>(
                    "run_id",
                    |m: &ArtifactGroupUploaderData| { &m.run_id },
                    |m: &mut ArtifactGroupUploaderData| { &mut m.run_id },
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_ptr_field_accessor::<_, ::protobuf::types::ProtobufTypeMessage<super::artifact::ArtifactId>>(
                    "id",
                    |m: &ArtifactGroupUploaderData| { &m.id },
                    |m: &mut ArtifactGroupUploaderData| { &mut m.id },
                ));
                fields.push(::protobuf::reflect::accessor::make_repeated_field_accessor::<_, ::protobuf::types::ProtobufTypeMessage<super::artifact::ArtifactId>>(
                    "ancestor_group_ids",
                    |m: &ArtifactGroupUploaderData| { &m.ancestor_group_ids },
                    |m: &mut ArtifactGroupUploaderData| { &mut m.ancestor_group_ids },
                ));
                ::protobuf::reflect::MessageDescriptor::new::<ArtifactGroupUploaderData>(
                    "ArtifactGroupUploaderData",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }

    fn default_instance() -> &'static ArtifactGroupUploaderData {
        static mut instance: ::protobuf::lazy::Lazy<ArtifactGroupUploaderData> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ArtifactGroupUploaderData,
        };
        unsafe {
            instance.get(ArtifactGroupUploaderData::new)
        }
    }
}

impl ::protobuf::Clear for ArtifactGroupUploaderData {
    fn clear(&mut self) {
        self.project_id.clear();
        self.run_id.clear();
        self.id.clear();
        self.ancestor_group_ids.clear();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for ArtifactGroupUploaderData {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for ArtifactGroupUploaderData {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct RunUploaderData {
    // message fields
    pub project_id: ::std::string::String,
    pub run_id: ::protobuf::SingularPtrField<super::run_id::RunId>,
    // special fields
    pub unknown_fields: ::protobuf::UnknownFields,
    pub cached_size: ::protobuf::CachedSize,
}

impl<'a> ::std::default::Default for &'a RunUploaderData {
    fn default() -> &'a RunUploaderData {
        <RunUploaderData as ::protobuf::Message>::default_instance()
    }
}

impl RunUploaderData {
    pub fn new() -> RunUploaderData {
        ::std::default::Default::default()
    }

    // string project_id = 1;


    pub fn get_project_id(&self) -> &str {
        &self.project_id
    }
    pub fn clear_project_id(&mut self) {
        self.project_id.clear();
    }

    // Param is passed by value, moved
    pub fn set_project_id(&mut self, v: ::std::string::String) {
        self.project_id = v;
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_project_id(&mut self) -> &mut ::std::string::String {
        &mut self.project_id
    }

    // Take field
    pub fn take_project_id(&mut self) -> ::std::string::String {
        ::std::mem::replace(&mut self.project_id, ::std::string::String::new())
    }

    // .observation_tools.proto.RunId run_id = 2;


    pub fn get_run_id(&self) -> &super::run_id::RunId {
        self.run_id.as_ref().unwrap_or_else(|| super::run_id::RunId::default_instance())
    }
    pub fn clear_run_id(&mut self) {
        self.run_id.clear();
    }

    pub fn has_run_id(&self) -> bool {
        self.run_id.is_some()
    }

    // Param is passed by value, moved
    pub fn set_run_id(&mut self, v: super::run_id::RunId) {
        self.run_id = ::protobuf::SingularPtrField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_run_id(&mut self) -> &mut super::run_id::RunId {
        if self.run_id.is_none() {
            self.run_id.set_default();
        }
        self.run_id.as_mut().unwrap()
    }

    // Take field
    pub fn take_run_id(&mut self) -> super::run_id::RunId {
        self.run_id.take().unwrap_or_else(|| super::run_id::RunId::new())
    }
}

impl ::protobuf::Message for RunUploaderData {
    fn is_initialized(&self) -> bool {
        for v in &self.run_id {
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
                1 => {
                    ::protobuf::rt::read_singular_proto3_string_into(wire_type, is, &mut self.project_id)?;
                },
                2 => {
                    ::protobuf::rt::read_singular_message_into(wire_type, is, &mut self.run_id)?;
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
        if !self.project_id.is_empty() {
            my_size += ::protobuf::rt::string_size(1, &self.project_id);
        }
        if let Some(ref v) = self.run_id.as_ref() {
            let len = v.compute_size();
            my_size += 1 + ::protobuf::rt::compute_raw_varint32_size(len) + len;
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream<'_>) -> ::protobuf::ProtobufResult<()> {
        if !self.project_id.is_empty() {
            os.write_string(1, &self.project_id)?;
        }
        if let Some(ref v) = self.run_id.as_ref() {
            os.write_tag(2, ::protobuf::wire_format::WireTypeLengthDelimited)?;
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

    fn new() -> RunUploaderData {
        RunUploaderData::new()
    }

    fn descriptor_static() -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                    "project_id",
                    |m: &RunUploaderData| { &m.project_id },
                    |m: &mut RunUploaderData| { &mut m.project_id },
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_ptr_field_accessor::<_, ::protobuf::types::ProtobufTypeMessage<super::run_id::RunId>>(
                    "run_id",
                    |m: &RunUploaderData| { &m.run_id },
                    |m: &mut RunUploaderData| { &mut m.run_id },
                ));
                ::protobuf::reflect::MessageDescriptor::new::<RunUploaderData>(
                    "RunUploaderData",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }

    fn default_instance() -> &'static RunUploaderData {
        static mut instance: ::protobuf::lazy::Lazy<RunUploaderData> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const RunUploaderData,
        };
        unsafe {
            instance.get(RunUploaderData::new)
        }
    }
}

impl ::protobuf::Clear for RunUploaderData {
    fn clear(&mut self) {
        self.project_id.clear();
        self.run_id.clear();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for RunUploaderData {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for RunUploaderData {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

static file_descriptor_proto_data: &'static [u8] = b"\
    \n#src/api/artifacts/api_context.proto\x12\x17observation_tools.proto\
    \x1a\x20src/api/artifacts/artifact.proto\x1a\x1esrc/api/artifacts/run_id\
    .proto\"\x85\x02\n\x19ArtifactGroupUploaderData\x12\x1d\n\nproject_id\
    \x18\x04\x20\x01(\tR\tprojectId\x125\n\x06run_id\x18\x05\x20\x01(\x0b2\
    \x1e.observation_tools.proto.RunIdR\x05runId\x123\n\x02id\x18\x03\x20\
    \x01(\x0b2#.observation_tools.proto.ArtifactIdR\x02id\x12Q\n\x12ancestor\
    _group_ids\x18\x06\x20\x03(\x0b2#.observation_tools.proto.ArtifactIdR\
    \x10ancestorGroupIdsJ\x04\x08\x01\x10\x02J\x04\x08\x02\x10\x03\"g\n\x0fR\
    unUploaderData\x12\x1d\n\nproject_id\x18\x01\x20\x01(\tR\tprojectId\x125\
    \n\x06run_id\x18\x02\x20\x01(\x0b2\x1e.observation_tools.proto.RunIdR\
    \x05runIdB\x1b\n\x17tools.observation.protoP\x01b\x06proto3\
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
