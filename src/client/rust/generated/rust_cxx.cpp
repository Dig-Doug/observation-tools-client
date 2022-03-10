#include <array>
#include <cassert>
#include <cstddef>
#include <cstdint>
#include <iterator>
#include <new>
#include <stdexcept>
#include <string>
#include <type_traits>
#include <utility>

namespace rust {
inline namespace cxxbridge1 {
// #include "rust/cxx.h"

#ifndef CXXBRIDGE1_PANIC
#define CXXBRIDGE1_PANIC
template <typename Exception>
void panic [[noreturn]] (const char *msg);
#endif // CXXBRIDGE1_PANIC

struct unsafe_bitcopy_t;

namespace {
template <typename T>
class impl;
} // namespace

template <typename T>
::std::size_t size_of();
template <typename T>
::std::size_t align_of();

#ifndef CXXBRIDGE1_RUST_STRING
#define CXXBRIDGE1_RUST_STRING
class String final {
public:
  String() noexcept;
  String(const String &) noexcept;
  String(String &&) noexcept;
  ~String() noexcept;

  String(const std::string &);
  String(const char *);
  String(const char *, std::size_t);
  String(const char16_t *);
  String(const char16_t *, std::size_t);

  static String lossy(const std::string &) noexcept;
  static String lossy(const char *) noexcept;
  static String lossy(const char *, std::size_t) noexcept;
  static String lossy(const char16_t *) noexcept;
  static String lossy(const char16_t *, std::size_t) noexcept;

  String &operator=(const String &) &noexcept;
  String &operator=(String &&) &noexcept;

  explicit operator std::string() const;

  const char *data() const noexcept;
  std::size_t size() const noexcept;
  std::size_t length() const noexcept;
  bool empty() const noexcept;

  const char *c_str() noexcept;

  std::size_t capacity() const noexcept;
  void reserve(size_t new_cap) noexcept;

  using iterator = char *;
  iterator begin() noexcept;
  iterator end() noexcept;

  using const_iterator = const char *;
  const_iterator begin() const noexcept;
  const_iterator end() const noexcept;
  const_iterator cbegin() const noexcept;
  const_iterator cend() const noexcept;

  bool operator==(const String &) const noexcept;
  bool operator!=(const String &) const noexcept;
  bool operator<(const String &) const noexcept;
  bool operator<=(const String &) const noexcept;
  bool operator>(const String &) const noexcept;
  bool operator>=(const String &) const noexcept;

  void swap(String &) noexcept;

  String(unsafe_bitcopy_t, const String &) noexcept;

private:
  struct lossy_t;
  String(lossy_t, const char *, std::size_t) noexcept;
  String(lossy_t, const char16_t *, std::size_t) noexcept;
  friend void swap(String &lhs, String &rhs) noexcept { lhs.swap(rhs); }

  std::array<std::uintptr_t, 3> repr;
};
#endif // CXXBRIDGE1_RUST_STRING

#ifndef CXXBRIDGE1_RUST_STR
#define CXXBRIDGE1_RUST_STR
class Str final {
public:
  Str() noexcept;
  Str(const String &) noexcept;
  Str(const std::string &);
  Str(const char *);
  Str(const char *, std::size_t);

  Str &operator=(const Str &) &noexcept = default;

  explicit operator std::string() const;

  const char *data() const noexcept;
  std::size_t size() const noexcept;
  std::size_t length() const noexcept;
  bool empty() const noexcept;

  Str(const Str &) noexcept = default;
  ~Str() noexcept = default;

  using iterator = const char *;
  using const_iterator = const char *;
  const_iterator begin() const noexcept;
  const_iterator end() const noexcept;
  const_iterator cbegin() const noexcept;
  const_iterator cend() const noexcept;

  bool operator==(const Str &) const noexcept;
  bool operator!=(const Str &) const noexcept;
  bool operator<(const Str &) const noexcept;
  bool operator<=(const Str &) const noexcept;
  bool operator>(const Str &) const noexcept;
  bool operator>=(const Str &) const noexcept;

  void swap(Str &) noexcept;

private:
  class uninit;
  Str(uninit) noexcept;
  friend impl<Str>;

  std::array<std::uintptr_t, 2> repr;
};
#endif // CXXBRIDGE1_RUST_STR

#ifndef CXXBRIDGE1_RUST_SLICE
#define CXXBRIDGE1_RUST_SLICE
namespace detail {
template <bool>
struct copy_assignable_if {};

template <>
struct copy_assignable_if<false> {
  copy_assignable_if() noexcept = default;
  copy_assignable_if(const copy_assignable_if &) noexcept = default;
  copy_assignable_if &operator=(const copy_assignable_if &) &noexcept = delete;
  copy_assignable_if &operator=(copy_assignable_if &&) &noexcept = default;
};
} // namespace detail

template <typename T>
class Slice final
    : private detail::copy_assignable_if<std::is_const<T>::value> {
public:
  using value_type = T;

  Slice() noexcept;
  Slice(T *, std::size_t count) noexcept;

  Slice &operator=(const Slice<T> &) &noexcept = default;
  Slice &operator=(Slice<T> &&) &noexcept = default;

  T *data() const noexcept;
  std::size_t size() const noexcept;
  std::size_t length() const noexcept;
  bool empty() const noexcept;

  T &operator[](std::size_t n) const noexcept;
  T &at(std::size_t n) const;
  T &front() const noexcept;
  T &back() const noexcept;

  Slice(const Slice<T> &) noexcept = default;
  ~Slice() noexcept = default;

  class iterator;
  iterator begin() const noexcept;
  iterator end() const noexcept;

  void swap(Slice &) noexcept;

private:
  class uninit;
  Slice(uninit) noexcept;
  friend impl<Slice>;
  friend void sliceInit(void *, const void *, std::size_t) noexcept;
  friend void *slicePtr(const void *) noexcept;
  friend std::size_t sliceLen(const void *) noexcept;

  std::array<std::uintptr_t, 2> repr;
};

template <typename T>
class Slice<T>::iterator final {
public:
  using iterator_category = std::random_access_iterator_tag;
  using value_type = T;
  using difference_type = std::ptrdiff_t;
  using pointer = typename std::add_pointer<T>::type;
  using reference = typename std::add_lvalue_reference<T>::type;

  reference operator*() const noexcept;
  pointer operator->() const noexcept;
  reference operator[](difference_type) const noexcept;

  iterator &operator++() noexcept;
  iterator operator++(int) noexcept;
  iterator &operator--() noexcept;
  iterator operator--(int) noexcept;

  iterator &operator+=(difference_type) noexcept;
  iterator &operator-=(difference_type) noexcept;
  iterator operator+(difference_type) const noexcept;
  iterator operator-(difference_type) const noexcept;
  difference_type operator-(const iterator &) const noexcept;

  bool operator==(const iterator &) const noexcept;
  bool operator!=(const iterator &) const noexcept;
  bool operator<(const iterator &) const noexcept;
  bool operator<=(const iterator &) const noexcept;
  bool operator>(const iterator &) const noexcept;
  bool operator>=(const iterator &) const noexcept;

private:
  friend class Slice;
  void *pos;
  std::size_t stride;
};

template <typename T>
Slice<T>::Slice() noexcept {
  sliceInit(this, reinterpret_cast<void *>(align_of<T>()), 0);
}

template <typename T>
Slice<T>::Slice(T *s, std::size_t count) noexcept {
  assert(s != nullptr || count == 0);
  sliceInit(this,
            s == nullptr && count == 0
                ? reinterpret_cast<void *>(align_of<T>())
                : const_cast<typename std::remove_const<T>::type *>(s),
            count);
}

template <typename T>
T *Slice<T>::data() const noexcept {
  return reinterpret_cast<T *>(slicePtr(this));
}

template <typename T>
std::size_t Slice<T>::size() const noexcept {
  return sliceLen(this);
}

template <typename T>
std::size_t Slice<T>::length() const noexcept {
  return this->size();
}

template <typename T>
bool Slice<T>::empty() const noexcept {
  return this->size() == 0;
}

template <typename T>
T &Slice<T>::operator[](std::size_t n) const noexcept {
  assert(n < this->size());
  auto ptr = static_cast<char *>(slicePtr(this)) + size_of<T>() * n;
  return *reinterpret_cast<T *>(ptr);
}

template <typename T>
T &Slice<T>::at(std::size_t n) const {
  if (n >= this->size()) {
    panic<std::out_of_range>("rust::Slice index out of range");
  }
  return (*this)[n];
}

template <typename T>
T &Slice<T>::front() const noexcept {
  assert(!this->empty());
  return (*this)[0];
}

template <typename T>
T &Slice<T>::back() const noexcept {
  assert(!this->empty());
  return (*this)[this->size() - 1];
}

template <typename T>
typename Slice<T>::iterator::reference
Slice<T>::iterator::operator*() const noexcept {
  return *static_cast<T *>(this->pos);
}

template <typename T>
typename Slice<T>::iterator::pointer
Slice<T>::iterator::operator->() const noexcept {
  return static_cast<T *>(this->pos);
}

template <typename T>
typename Slice<T>::iterator::reference Slice<T>::iterator::operator[](
    typename Slice<T>::iterator::difference_type n) const noexcept {
  auto ptr = static_cast<char *>(this->pos) + this->stride * n;
  return *reinterpret_cast<T *>(ptr);
}

template <typename T>
typename Slice<T>::iterator &Slice<T>::iterator::operator++() noexcept {
  this->pos = static_cast<char *>(this->pos) + this->stride;
  return *this;
}

template <typename T>
typename Slice<T>::iterator Slice<T>::iterator::operator++(int) noexcept {
  auto ret = iterator(*this);
  this->pos = static_cast<char *>(this->pos) + this->stride;
  return ret;
}

template <typename T>
typename Slice<T>::iterator &Slice<T>::iterator::operator--() noexcept {
  this->pos = static_cast<char *>(this->pos) - this->stride;
  return *this;
}

template <typename T>
typename Slice<T>::iterator Slice<T>::iterator::operator--(int) noexcept {
  auto ret = iterator(*this);
  this->pos = static_cast<char *>(this->pos) - this->stride;
  return ret;
}

template <typename T>
typename Slice<T>::iterator &Slice<T>::iterator::operator+=(
    typename Slice<T>::iterator::difference_type n) noexcept {
  this->pos = static_cast<char *>(this->pos) + this->stride * n;
  return *this;
}

template <typename T>
typename Slice<T>::iterator &Slice<T>::iterator::operator-=(
    typename Slice<T>::iterator::difference_type n) noexcept {
  this->pos = static_cast<char *>(this->pos) - this->stride * n;
  return *this;
}

template <typename T>
typename Slice<T>::iterator Slice<T>::iterator::operator+(
    typename Slice<T>::iterator::difference_type n) const noexcept {
  auto ret = iterator(*this);
  ret.pos = static_cast<char *>(this->pos) + this->stride * n;
  return ret;
}

template <typename T>
typename Slice<T>::iterator Slice<T>::iterator::operator-(
    typename Slice<T>::iterator::difference_type n) const noexcept {
  auto ret = iterator(*this);
  ret.pos = static_cast<char *>(this->pos) - this->stride * n;
  return ret;
}

template <typename T>
typename Slice<T>::iterator::difference_type
Slice<T>::iterator::operator-(const iterator &other) const noexcept {
  auto diff = std::distance(static_cast<char *>(other.pos),
                            static_cast<char *>(this->pos));
  return diff / this->stride;
}

template <typename T>
bool Slice<T>::iterator::operator==(const iterator &other) const noexcept {
  return this->pos == other.pos;
}

template <typename T>
bool Slice<T>::iterator::operator!=(const iterator &other) const noexcept {
  return this->pos != other.pos;
}

template <typename T>
bool Slice<T>::iterator::operator<(const iterator &other) const noexcept {
  return this->pos < other.pos;
}

template <typename T>
bool Slice<T>::iterator::operator<=(const iterator &other) const noexcept {
  return this->pos <= other.pos;
}

template <typename T>
bool Slice<T>::iterator::operator>(const iterator &other) const noexcept {
  return this->pos > other.pos;
}

template <typename T>
bool Slice<T>::iterator::operator>=(const iterator &other) const noexcept {
  return this->pos >= other.pos;
}

template <typename T>
typename Slice<T>::iterator Slice<T>::begin() const noexcept {
  iterator it;
  it.pos = slicePtr(this);
  it.stride = size_of<T>();
  return it;
}

template <typename T>
typename Slice<T>::iterator Slice<T>::end() const noexcept {
  iterator it = this->begin();
  it.pos = static_cast<char *>(it.pos) + it.stride * this->size();
  return it;
}

template <typename T>
void Slice<T>::swap(Slice &rhs) noexcept {
  std::swap(*this, rhs);
}
#endif // CXXBRIDGE1_RUST_SLICE

#ifndef CXXBRIDGE1_RUST_BOX
#define CXXBRIDGE1_RUST_BOX
template <typename T>
class Box final {
public:
  using element_type = T;
  using const_pointer =
      typename std::add_pointer<typename std::add_const<T>::type>::type;
  using pointer = typename std::add_pointer<T>::type;

  Box() = delete;
  Box(Box &&) noexcept;
  ~Box() noexcept;

  explicit Box(const T &);
  explicit Box(T &&);

  Box &operator=(Box &&) &noexcept;

  const T *operator->() const noexcept;
  const T &operator*() const noexcept;
  T *operator->() noexcept;
  T &operator*() noexcept;

  template <typename... Fields>
  static Box in_place(Fields &&...);

  void swap(Box &) noexcept;

  static Box from_raw(T *) noexcept;

  T *into_raw() noexcept;

  /* Deprecated */ using value_type = element_type;

private:
  class uninit;
  class allocation;
  Box(uninit) noexcept;
  void drop() noexcept;

  friend void swap(Box &lhs, Box &rhs) noexcept { lhs.swap(rhs); }

  T *ptr;
};

template <typename T>
class Box<T>::uninit {};

template <typename T>
class Box<T>::allocation {
  static T *alloc() noexcept;
  static void dealloc(T *) noexcept;

public:
  allocation() noexcept : ptr(alloc()) {}
  ~allocation() noexcept {
    if (this->ptr) {
      dealloc(this->ptr);
    }
  }
  T *ptr;
};

template <typename T>
Box<T>::Box(Box &&other) noexcept : ptr(other.ptr) {
  other.ptr = nullptr;
}

template <typename T>
Box<T>::Box(const T &val) {
  allocation alloc;
  ::new (alloc.ptr) T(val);
  this->ptr = alloc.ptr;
  alloc.ptr = nullptr;
}

template <typename T>
Box<T>::Box(T &&val) {
  allocation alloc;
  ::new (alloc.ptr) T(std::move(val));
  this->ptr = alloc.ptr;
  alloc.ptr = nullptr;
}

template <typename T>
Box<T>::~Box() noexcept {
  if (this->ptr) {
    this->drop();
  }
}

template <typename T>
Box<T> &Box<T>::operator=(Box &&other) &noexcept {
  if (this->ptr) {
    this->drop();
  }
  this->ptr = other.ptr;
  other.ptr = nullptr;
  return *this;
}

template <typename T>
const T *Box<T>::operator->() const noexcept {
  return this->ptr;
}

template <typename T>
const T &Box<T>::operator*() const noexcept {
  return *this->ptr;
}

template <typename T>
T *Box<T>::operator->() noexcept {
  return this->ptr;
}

template <typename T>
T &Box<T>::operator*() noexcept {
  return *this->ptr;
}

template <typename T>
template <typename... Fields>
Box<T> Box<T>::in_place(Fields &&...fields) {
  allocation alloc;
  auto ptr = alloc.ptr;
  ::new (ptr) T{std::forward<Fields>(fields)...};
  alloc.ptr = nullptr;
  return from_raw(ptr);
}

template <typename T>
void Box<T>::swap(Box &rhs) noexcept {
  using std::swap;
  swap(this->ptr, rhs.ptr);
}

template <typename T>
Box<T> Box<T>::from_raw(T *raw) noexcept {
  Box box = uninit{};
  box.ptr = raw;
  return box;
}

template <typename T>
T *Box<T>::into_raw() noexcept {
  T *raw = this->ptr;
  this->ptr = nullptr;
  return raw;
}

template <typename T>
Box<T>::Box(uninit) noexcept {}
#endif // CXXBRIDGE1_RUST_BOX

#ifndef CXXBRIDGE1_RUST_OPAQUE
#define CXXBRIDGE1_RUST_OPAQUE
class Opaque {
public:
  Opaque() = delete;
  Opaque(const Opaque &) = delete;
  ~Opaque() = delete;
};
#endif // CXXBRIDGE1_RUST_OPAQUE

#ifndef CXXBRIDGE1_IS_COMPLETE
#define CXXBRIDGE1_IS_COMPLETE
namespace detail {
namespace {
template <typename T, typename = std::size_t>
struct is_complete : std::false_type {};
template <typename T>
struct is_complete<T, decltype(sizeof(T))> : std::true_type {};
} // namespace
} // namespace detail
#endif // CXXBRIDGE1_IS_COMPLETE

#ifndef CXXBRIDGE1_LAYOUT
#define CXXBRIDGE1_LAYOUT
class layout {
  template <typename T>
  friend std::size_t size_of();
  template <typename T>
  friend std::size_t align_of();
  template <typename T>
  static typename std::enable_if<std::is_base_of<Opaque, T>::value,
                                 std::size_t>::type
  do_size_of() {
    return T::layout::size();
  }
  template <typename T>
  static typename std::enable_if<!std::is_base_of<Opaque, T>::value,
                                 std::size_t>::type
  do_size_of() {
    return sizeof(T);
  }
  template <typename T>
  static
      typename std::enable_if<detail::is_complete<T>::value, std::size_t>::type
      size_of() {
    return do_size_of<T>();
  }
  template <typename T>
  static typename std::enable_if<std::is_base_of<Opaque, T>::value,
                                 std::size_t>::type
  do_align_of() {
    return T::layout::align();
  }
  template <typename T>
  static typename std::enable_if<!std::is_base_of<Opaque, T>::value,
                                 std::size_t>::type
  do_align_of() {
    return alignof(T);
  }
  template <typename T>
  static
      typename std::enable_if<detail::is_complete<T>::value, std::size_t>::type
      align_of() {
    return do_align_of<T>();
  }
};

template <typename T>
std::size_t size_of() {
  return layout::size_of<T>();
}

template <typename T>
std::size_t align_of() {
  return layout::align_of<T>();
}
#endif // CXXBRIDGE1_LAYOUT

class Str::uninit {};
inline Str::Str(uninit) noexcept {}

namespace detail {
template <typename T, typename = void *>
struct operator_new {
  void *operator()(::std::size_t sz) { return ::operator new(sz); }
};

template <typename T>
struct operator_new<T, decltype(T::operator new(sizeof(T)))> {
  void *operator()(::std::size_t sz) { return T::operator new(sz); }
};
} // namespace detail

template <typename T>
union MaybeUninit {
  T value;
  void *operator new(::std::size_t sz) { return detail::operator_new<T>{}(sz); }
  MaybeUninit() {}
  ~MaybeUninit() {}
};

namespace {
namespace repr {
using Fat = ::std::array<::std::uintptr_t, 2>;
} // namespace repr

template <>
class impl<Str> final {
public:
  static Str new_unchecked(repr::Fat repr) noexcept {
    Str str = Str::uninit{};
    str.repr = repr;
    return str;
  }
};
} // namespace
} // namespace cxxbridge1
} // namespace rust

struct Client;
struct RunUploader;
struct UserMetadataBuilder;
struct RunStageUploader;
struct GenericArtifactUploader;
struct ArtifactUploader2d;
struct ArtifactUploader3d;

#ifndef CXXBRIDGE1_STRUCT_Client
#define CXXBRIDGE1_STRUCT_Client
struct Client final : public ::rust::Opaque {
  ::rust::Box<::RunUploader> ffi_create_run() const noexcept;
  ::rust::Box<::RunStageUploader> ffi_deserialize_run_stage(::rust::String serialized) const noexcept;
  ~Client() = delete;

private:
  friend ::rust::layout;
  struct layout {
    static ::std::size_t size() noexcept;
    static ::std::size_t align() noexcept;
  };
};
#endif // CXXBRIDGE1_STRUCT_Client

#ifndef CXXBRIDGE1_STRUCT_RunUploader
#define CXXBRIDGE1_STRUCT_RunUploader
struct RunUploader final : public ::rust::Opaque {
  ::rust::Str viewer_url() const noexcept;
  ::rust::Box<::RunStageUploader> ffi_create_initial_run_stage(const ::UserMetadataBuilder &metadata) const noexcept;
  ~RunUploader() = delete;

private:
  friend ::rust::layout;
  struct layout {
    static ::std::size_t size() noexcept;
    static ::std::size_t align() noexcept;
  };
};
#endif // CXXBRIDGE1_STRUCT_RunUploader

#ifndef CXXBRIDGE1_STRUCT_UserMetadataBuilder
#define CXXBRIDGE1_STRUCT_UserMetadataBuilder
struct UserMetadataBuilder final : public ::rust::Opaque {
  ::UserMetadataBuilder &add_metadata(::rust::String key, ::rust::String value) noexcept;
  ~UserMetadataBuilder() = delete;

private:
  friend ::rust::layout;
  struct layout {
    static ::std::size_t size() noexcept;
    static ::std::size_t align() noexcept;
  };
};
#endif // CXXBRIDGE1_STRUCT_UserMetadataBuilder

#ifndef CXXBRIDGE1_STRUCT_RunStageUploader
#define CXXBRIDGE1_STRUCT_RunStageUploader
struct RunStageUploader final : public ::rust::Opaque {
  ::rust::Box<::GenericArtifactUploader> ffi_child_uploader(const ::UserMetadataBuilder &metadata) const noexcept;
  ::rust::Box<::ArtifactUploader2d> ffi_child_uploader_2d(const ::UserMetadataBuilder &metadata) const noexcept;
  ::rust::String ffi_upload(const ::UserMetadataBuilder &metadata, ::rust::Slice<const ::std::uint8_t> data) const noexcept;
  ~RunStageUploader() = delete;

private:
  friend ::rust::layout;
  struct layout {
    static ::std::size_t size() noexcept;
    static ::std::size_t align() noexcept;
  };
};
#endif // CXXBRIDGE1_STRUCT_RunStageUploader

#ifndef CXXBRIDGE1_STRUCT_GenericArtifactUploader
#define CXXBRIDGE1_STRUCT_GenericArtifactUploader
struct GenericArtifactUploader final : public ::rust::Opaque {
  ::rust::Box<::GenericArtifactUploader> ffi_child_uploader(const ::UserMetadataBuilder &metadata) const noexcept;
  ::rust::Box<::ArtifactUploader2d> ffi_child_uploader_2d(const ::UserMetadataBuilder &metadata) const noexcept;
  ::rust::Box<::ArtifactUploader3d> ffi_child_uploader_3d(const ::UserMetadataBuilder &metadata, ::rust::Slice<const ::std::uint8_t> transform3_bytes) const noexcept;
  ::rust::String ffi_upload(const ::UserMetadataBuilder &metadata, ::rust::Slice<const ::std::uint8_t> data) const noexcept;
  ~GenericArtifactUploader() = delete;

private:
  friend ::rust::layout;
  struct layout {
    static ::std::size_t size() noexcept;
    static ::std::size_t align() noexcept;
  };
};
#endif // CXXBRIDGE1_STRUCT_GenericArtifactUploader

#ifndef CXXBRIDGE1_STRUCT_ArtifactUploader2d
#define CXXBRIDGE1_STRUCT_ArtifactUploader2d
struct ArtifactUploader2d final : public ::rust::Opaque {
  ::rust::String ffi_upload(const ::UserMetadataBuilder &metadata, ::rust::Slice<const ::std::uint8_t> data) const noexcept;
  ~ArtifactUploader2d() = delete;

private:
  friend ::rust::layout;
  struct layout {
    static ::std::size_t size() noexcept;
    static ::std::size_t align() noexcept;
  };
};
#endif // CXXBRIDGE1_STRUCT_ArtifactUploader2d

#ifndef CXXBRIDGE1_STRUCT_ArtifactUploader3d
#define CXXBRIDGE1_STRUCT_ArtifactUploader3d
struct ArtifactUploader3d final : public ::rust::Opaque {
  ::rust::Box<::ArtifactUploader2d> ffi_child_uploader_2d(const ::UserMetadataBuilder &metadata, ::rust::Slice<const ::std::uint8_t> to_3d_transform) const noexcept;
  ::rust::String ffi_upload(const ::UserMetadataBuilder &metadata, ::rust::Slice<const ::std::uint8_t> data) const noexcept;
  ~ArtifactUploader3d() = delete;

private:
  friend ::rust::layout;
  struct layout {
    static ::std::size_t size() noexcept;
    static ::std::size_t align() noexcept;
  };
};
#endif // CXXBRIDGE1_STRUCT_ArtifactUploader3d

extern "C" {
::std::size_t cxxbridge1$Client$operator$sizeof() noexcept;
::std::size_t cxxbridge1$Client$operator$alignof() noexcept;

::Client *cxxbridge1$ffi_new_client(::rust::String *project_id) noexcept;

::RunUploader *cxxbridge1$Client$ffi_create_run(const ::Client &self) noexcept;

::RunStageUploader *cxxbridge1$Client$ffi_deserialize_run_stage(const ::Client &self, ::rust::String *serialized) noexcept;
::std::size_t cxxbridge1$RunUploader$operator$sizeof() noexcept;
::std::size_t cxxbridge1$RunUploader$operator$alignof() noexcept;

::rust::repr::Fat cxxbridge1$RunUploader$viewer_url(const ::RunUploader &self) noexcept;

::RunStageUploader *cxxbridge1$RunUploader$ffi_create_initial_run_stage(const ::RunUploader &self, const ::UserMetadataBuilder &metadata) noexcept;
::std::size_t cxxbridge1$UserMetadataBuilder$operator$sizeof() noexcept;
::std::size_t cxxbridge1$UserMetadataBuilder$operator$alignof() noexcept;

::UserMetadataBuilder *cxxbridge1$new_user_metadata(::rust::String *name) noexcept;

::UserMetadataBuilder *cxxbridge1$UserMetadataBuilder$add_metadata(::UserMetadataBuilder &self, ::rust::String *key, ::rust::String *value) noexcept;
::std::size_t cxxbridge1$RunStageUploader$operator$sizeof() noexcept;
::std::size_t cxxbridge1$RunStageUploader$operator$alignof() noexcept;

::GenericArtifactUploader *cxxbridge1$RunStageUploader$ffi_child_uploader(const ::RunStageUploader &self, const ::UserMetadataBuilder &metadata) noexcept;

::ArtifactUploader2d *cxxbridge1$RunStageUploader$ffi_child_uploader_2d(const ::RunStageUploader &self, const ::UserMetadataBuilder &metadata) noexcept;

void cxxbridge1$RunStageUploader$ffi_upload(const ::RunStageUploader &self, const ::UserMetadataBuilder &metadata, ::rust::Slice<const ::std::uint8_t> data, ::rust::String *return$) noexcept;
::std::size_t cxxbridge1$GenericArtifactUploader$operator$sizeof() noexcept;
::std::size_t cxxbridge1$GenericArtifactUploader$operator$alignof() noexcept;

::GenericArtifactUploader *cxxbridge1$GenericArtifactUploader$ffi_child_uploader(const ::GenericArtifactUploader &self, const ::UserMetadataBuilder &metadata) noexcept;

::ArtifactUploader2d *cxxbridge1$GenericArtifactUploader$ffi_child_uploader_2d(const ::GenericArtifactUploader &self, const ::UserMetadataBuilder &metadata) noexcept;

::ArtifactUploader3d *cxxbridge1$GenericArtifactUploader$ffi_child_uploader_3d(const ::GenericArtifactUploader &self, const ::UserMetadataBuilder &metadata, ::rust::Slice<const ::std::uint8_t> transform3_bytes) noexcept;

void cxxbridge1$GenericArtifactUploader$ffi_upload(const ::GenericArtifactUploader &self, const ::UserMetadataBuilder &metadata, ::rust::Slice<const ::std::uint8_t> data, ::rust::String *return$) noexcept;
::std::size_t cxxbridge1$ArtifactUploader2d$operator$sizeof() noexcept;
::std::size_t cxxbridge1$ArtifactUploader2d$operator$alignof() noexcept;

void cxxbridge1$ArtifactUploader2d$ffi_upload(const ::ArtifactUploader2d &self, const ::UserMetadataBuilder &metadata, ::rust::Slice<const ::std::uint8_t> data, ::rust::String *return$) noexcept;
::std::size_t cxxbridge1$ArtifactUploader3d$operator$sizeof() noexcept;
::std::size_t cxxbridge1$ArtifactUploader3d$operator$alignof() noexcept;

::ArtifactUploader2d *cxxbridge1$ArtifactUploader3d$ffi_child_uploader_2d(const ::ArtifactUploader3d &self, const ::UserMetadataBuilder &metadata, ::rust::Slice<const ::std::uint8_t> to_3d_transform) noexcept;

void cxxbridge1$ArtifactUploader3d$ffi_upload(const ::ArtifactUploader3d &self, const ::UserMetadataBuilder &metadata, ::rust::Slice<const ::std::uint8_t> data, ::rust::String *return$) noexcept;

::GenericArtifactUploader *cxxbridge1$ffi_get_current_group() noexcept;
} // extern "C"

::std::size_t Client::layout::size() noexcept {
  return cxxbridge1$Client$operator$sizeof();
}

::std::size_t Client::layout::align() noexcept {
  return cxxbridge1$Client$operator$alignof();
}

::rust::Box<::Client> ffi_new_client(::rust::String project_id) noexcept {
  return ::rust::Box<::Client>::from_raw(cxxbridge1$ffi_new_client(&project_id));
}

::rust::Box<::RunUploader> Client::ffi_create_run() const noexcept {
  return ::rust::Box<::RunUploader>::from_raw(cxxbridge1$Client$ffi_create_run(*this));
}

::rust::Box<::RunStageUploader> Client::ffi_deserialize_run_stage(::rust::String serialized) const noexcept {
  return ::rust::Box<::RunStageUploader>::from_raw(cxxbridge1$Client$ffi_deserialize_run_stage(*this, &serialized));
}

::std::size_t RunUploader::layout::size() noexcept {
  return cxxbridge1$RunUploader$operator$sizeof();
}

::std::size_t RunUploader::layout::align() noexcept {
  return cxxbridge1$RunUploader$operator$alignof();
}

::rust::Str RunUploader::viewer_url() const noexcept {
  return ::rust::impl<::rust::Str>::new_unchecked(cxxbridge1$RunUploader$viewer_url(*this));
}

::rust::Box<::RunStageUploader> RunUploader::ffi_create_initial_run_stage(const ::UserMetadataBuilder &metadata) const noexcept {
  return ::rust::Box<::RunStageUploader>::from_raw(cxxbridge1$RunUploader$ffi_create_initial_run_stage(*this, metadata));
}

::std::size_t UserMetadataBuilder::layout::size() noexcept {
  return cxxbridge1$UserMetadataBuilder$operator$sizeof();
}

::std::size_t UserMetadataBuilder::layout::align() noexcept {
  return cxxbridge1$UserMetadataBuilder$operator$alignof();
}

::rust::Box<::UserMetadataBuilder> new_user_metadata(::rust::String name) noexcept {
  return ::rust::Box<::UserMetadataBuilder>::from_raw(cxxbridge1$new_user_metadata(&name));
}

::UserMetadataBuilder &UserMetadataBuilder::add_metadata(::rust::String key, ::rust::String value) noexcept {
  return *cxxbridge1$UserMetadataBuilder$add_metadata(*this, &key, &value);
}

::std::size_t RunStageUploader::layout::size() noexcept {
  return cxxbridge1$RunStageUploader$operator$sizeof();
}

::std::size_t RunStageUploader::layout::align() noexcept {
  return cxxbridge1$RunStageUploader$operator$alignof();
}

::rust::Box<::GenericArtifactUploader> RunStageUploader::ffi_child_uploader(const ::UserMetadataBuilder &metadata) const noexcept {
  return ::rust::Box<::GenericArtifactUploader>::from_raw(cxxbridge1$RunStageUploader$ffi_child_uploader(*this, metadata));
}

::rust::Box<::ArtifactUploader2d> RunStageUploader::ffi_child_uploader_2d(const ::UserMetadataBuilder &metadata) const noexcept {
  return ::rust::Box<::ArtifactUploader2d>::from_raw(cxxbridge1$RunStageUploader$ffi_child_uploader_2d(*this, metadata));
}

::rust::String RunStageUploader::ffi_upload(const ::UserMetadataBuilder &metadata, ::rust::Slice<const ::std::uint8_t> data) const noexcept {
  ::rust::MaybeUninit<::rust::String> return$;
  cxxbridge1$RunStageUploader$ffi_upload(*this, metadata, data, &return$.value);
  return ::std::move(return$.value);
}

::std::size_t GenericArtifactUploader::layout::size() noexcept {
  return cxxbridge1$GenericArtifactUploader$operator$sizeof();
}

::std::size_t GenericArtifactUploader::layout::align() noexcept {
  return cxxbridge1$GenericArtifactUploader$operator$alignof();
}

::rust::Box<::GenericArtifactUploader> GenericArtifactUploader::ffi_child_uploader(const ::UserMetadataBuilder &metadata) const noexcept {
  return ::rust::Box<::GenericArtifactUploader>::from_raw(cxxbridge1$GenericArtifactUploader$ffi_child_uploader(*this, metadata));
}

::rust::Box<::ArtifactUploader2d> GenericArtifactUploader::ffi_child_uploader_2d(const ::UserMetadataBuilder &metadata) const noexcept {
  return ::rust::Box<::ArtifactUploader2d>::from_raw(cxxbridge1$GenericArtifactUploader$ffi_child_uploader_2d(*this, metadata));
}

::rust::Box<::ArtifactUploader3d> GenericArtifactUploader::ffi_child_uploader_3d(const ::UserMetadataBuilder &metadata, ::rust::Slice<const ::std::uint8_t> transform3_bytes) const noexcept {
  return ::rust::Box<::ArtifactUploader3d>::from_raw(cxxbridge1$GenericArtifactUploader$ffi_child_uploader_3d(*this, metadata, transform3_bytes));
}

::rust::String GenericArtifactUploader::ffi_upload(const ::UserMetadataBuilder &metadata, ::rust::Slice<const ::std::uint8_t> data) const noexcept {
  ::rust::MaybeUninit<::rust::String> return$;
  cxxbridge1$GenericArtifactUploader$ffi_upload(*this, metadata, data, &return$.value);
  return ::std::move(return$.value);
}

::std::size_t ArtifactUploader2d::layout::size() noexcept {
  return cxxbridge1$ArtifactUploader2d$operator$sizeof();
}

::std::size_t ArtifactUploader2d::layout::align() noexcept {
  return cxxbridge1$ArtifactUploader2d$operator$alignof();
}

::rust::String ArtifactUploader2d::ffi_upload(const ::UserMetadataBuilder &metadata, ::rust::Slice<const ::std::uint8_t> data) const noexcept {
  ::rust::MaybeUninit<::rust::String> return$;
  cxxbridge1$ArtifactUploader2d$ffi_upload(*this, metadata, data, &return$.value);
  return ::std::move(return$.value);
}

::std::size_t ArtifactUploader3d::layout::size() noexcept {
  return cxxbridge1$ArtifactUploader3d$operator$sizeof();
}

::std::size_t ArtifactUploader3d::layout::align() noexcept {
  return cxxbridge1$ArtifactUploader3d$operator$alignof();
}

::rust::Box<::ArtifactUploader2d> ArtifactUploader3d::ffi_child_uploader_2d(const ::UserMetadataBuilder &metadata, ::rust::Slice<const ::std::uint8_t> to_3d_transform) const noexcept {
  return ::rust::Box<::ArtifactUploader2d>::from_raw(cxxbridge1$ArtifactUploader3d$ffi_child_uploader_2d(*this, metadata, to_3d_transform));
}

::rust::String ArtifactUploader3d::ffi_upload(const ::UserMetadataBuilder &metadata, ::rust::Slice<const ::std::uint8_t> data) const noexcept {
  ::rust::MaybeUninit<::rust::String> return$;
  cxxbridge1$ArtifactUploader3d$ffi_upload(*this, metadata, data, &return$.value);
  return ::std::move(return$.value);
}

::rust::Box<::GenericArtifactUploader> ffi_get_current_group() noexcept {
  return ::rust::Box<::GenericArtifactUploader>::from_raw(cxxbridge1$ffi_get_current_group());
}

extern "C" {
::Client *cxxbridge1$box$Client$alloc() noexcept;
void cxxbridge1$box$Client$dealloc(::Client *) noexcept;
void cxxbridge1$box$Client$drop(::rust::Box<::Client> *ptr) noexcept;

::RunUploader *cxxbridge1$box$RunUploader$alloc() noexcept;
void cxxbridge1$box$RunUploader$dealloc(::RunUploader *) noexcept;
void cxxbridge1$box$RunUploader$drop(::rust::Box<::RunUploader> *ptr) noexcept;

::RunStageUploader *cxxbridge1$box$RunStageUploader$alloc() noexcept;
void cxxbridge1$box$RunStageUploader$dealloc(::RunStageUploader *) noexcept;
void cxxbridge1$box$RunStageUploader$drop(::rust::Box<::RunStageUploader> *ptr) noexcept;

::UserMetadataBuilder *cxxbridge1$box$UserMetadataBuilder$alloc() noexcept;
void cxxbridge1$box$UserMetadataBuilder$dealloc(::UserMetadataBuilder *) noexcept;
void cxxbridge1$box$UserMetadataBuilder$drop(::rust::Box<::UserMetadataBuilder> *ptr) noexcept;

::GenericArtifactUploader *cxxbridge1$box$GenericArtifactUploader$alloc() noexcept;
void cxxbridge1$box$GenericArtifactUploader$dealloc(::GenericArtifactUploader *) noexcept;
void cxxbridge1$box$GenericArtifactUploader$drop(::rust::Box<::GenericArtifactUploader> *ptr) noexcept;

::ArtifactUploader2d *cxxbridge1$box$ArtifactUploader2d$alloc() noexcept;
void cxxbridge1$box$ArtifactUploader2d$dealloc(::ArtifactUploader2d *) noexcept;
void cxxbridge1$box$ArtifactUploader2d$drop(::rust::Box<::ArtifactUploader2d> *ptr) noexcept;

::ArtifactUploader3d *cxxbridge1$box$ArtifactUploader3d$alloc() noexcept;
void cxxbridge1$box$ArtifactUploader3d$dealloc(::ArtifactUploader3d *) noexcept;
void cxxbridge1$box$ArtifactUploader3d$drop(::rust::Box<::ArtifactUploader3d> *ptr) noexcept;
} // extern "C"

namespace rust {
inline namespace cxxbridge1 {
template <>
::Client *Box<::Client>::allocation::alloc() noexcept {
  return cxxbridge1$box$Client$alloc();
}
template <>
void Box<::Client>::allocation::dealloc(::Client *ptr) noexcept {
  cxxbridge1$box$Client$dealloc(ptr);
}
template <>
void Box<::Client>::drop() noexcept {
  cxxbridge1$box$Client$drop(this);
}
template <>
::RunUploader *Box<::RunUploader>::allocation::alloc() noexcept {
  return cxxbridge1$box$RunUploader$alloc();
}
template <>
void Box<::RunUploader>::allocation::dealloc(::RunUploader *ptr) noexcept {
  cxxbridge1$box$RunUploader$dealloc(ptr);
}
template <>
void Box<::RunUploader>::drop() noexcept {
  cxxbridge1$box$RunUploader$drop(this);
}
template <>
::RunStageUploader *Box<::RunStageUploader>::allocation::alloc() noexcept {
  return cxxbridge1$box$RunStageUploader$alloc();
}
template <>
void Box<::RunStageUploader>::allocation::dealloc(::RunStageUploader *ptr) noexcept {
  cxxbridge1$box$RunStageUploader$dealloc(ptr);
}
template <>
void Box<::RunStageUploader>::drop() noexcept {
  cxxbridge1$box$RunStageUploader$drop(this);
}
template <>
::UserMetadataBuilder *Box<::UserMetadataBuilder>::allocation::alloc() noexcept {
  return cxxbridge1$box$UserMetadataBuilder$alloc();
}
template <>
void Box<::UserMetadataBuilder>::allocation::dealloc(::UserMetadataBuilder *ptr) noexcept {
  cxxbridge1$box$UserMetadataBuilder$dealloc(ptr);
}
template <>
void Box<::UserMetadataBuilder>::drop() noexcept {
  cxxbridge1$box$UserMetadataBuilder$drop(this);
}
template <>
::GenericArtifactUploader *Box<::GenericArtifactUploader>::allocation::alloc() noexcept {
  return cxxbridge1$box$GenericArtifactUploader$alloc();
}
template <>
void Box<::GenericArtifactUploader>::allocation::dealloc(::GenericArtifactUploader *ptr) noexcept {
  cxxbridge1$box$GenericArtifactUploader$dealloc(ptr);
}
template <>
void Box<::GenericArtifactUploader>::drop() noexcept {
  cxxbridge1$box$GenericArtifactUploader$drop(this);
}
template <>
::ArtifactUploader2d *Box<::ArtifactUploader2d>::allocation::alloc() noexcept {
  return cxxbridge1$box$ArtifactUploader2d$alloc();
}
template <>
void Box<::ArtifactUploader2d>::allocation::dealloc(::ArtifactUploader2d *ptr) noexcept {
  cxxbridge1$box$ArtifactUploader2d$dealloc(ptr);
}
template <>
void Box<::ArtifactUploader2d>::drop() noexcept {
  cxxbridge1$box$ArtifactUploader2d$drop(this);
}
template <>
::ArtifactUploader3d *Box<::ArtifactUploader3d>::allocation::alloc() noexcept {
  return cxxbridge1$box$ArtifactUploader3d$alloc();
}
template <>
void Box<::ArtifactUploader3d>::allocation::dealloc(::ArtifactUploader3d *ptr) noexcept {
  cxxbridge1$box$ArtifactUploader3d$dealloc(ptr);
}
template <>
void Box<::ArtifactUploader3d>::drop() noexcept {
  cxxbridge1$box$ArtifactUploader3d$drop(this);
}
} // namespace cxxbridge1
} // namespace rust
