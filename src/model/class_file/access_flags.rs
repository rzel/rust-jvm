#[allow(non_camel_case_types)]
pub type u2 = super::u2;

/// Values of access flags for a class or interface.
pub mod class_access_flags {
    #[allow(non_camel_case_types)]
    pub type access_flag = super::u2;
    #[allow(non_camel_case_types)]
    pub type t = access_flag;

    /// Declared `public`; may be accessed from outside its package.
    pub const ACC_PUBLIC: t = 0x0001;
    /// Declared `final`; no subclasses allowed.
    pub const ACC_FINAL: t = 0x0010;
    /// Treat superclass methods specially when invoked by the _invokespecial_
    /// instruction.
    pub const ACC_SUPER: t = 0x0020;
    /// Is an interface, not a class.
    pub const ACC_INTERFACE: t = 0x0200;
    /// Declared `abstract`; must not be instantiated.
    pub const ACC_ABSTRACT: t = 0x0400;
    /// Declared synthetic; not present in the source code.
    pub const ACC_SYNTHETIC: t = 0x1000;
    /// Declared as an annotation type.
    pub const ACC_ANNOTATION: t = 0x2000;
    /// Declared as an `enum` type.
    pub const ACC_ENUM: t = 0x4000;
}

/// Values of access flags for an inner class.
pub mod inner_class_access_flags {
    #[allow(non_camel_case_types)]
    pub type access_flag = super::u2;
    #[allow(non_camel_case_types)]
    pub type t = access_flag;

    /// Marked or implicitly `public` in source.
    pub const ACC_PUBLIC: t = 0x0001;
    /// Marked `private` in source.
    pub const ACC_PRIVATE: t = 0x0002;
    /// Marked `protected` in source.
    pub const ACC_PROTECTED: t = 0x0004;
    /// Marked or implicitly `static` in source.
    pub const ACC_STATIC: t = 0x0008;
    /// Marked `final` in source.
    pub const ACC_FINAL: t = 0x0010;
    /// Was an `interface` in source.
    pub const ACC_INTERFACE: t = 0x0200;
    /// Marked or implicitly `abstract` in source.
    pub const ACC_ABSTRACT: t = 0x0400;
    /// Declared synthetic; not present in the source code.
    pub const ACC_SYNTHETIC: t = 0x1000;
    /// Declared as an annotation type.
    pub const ACC_ANNOTATION: t = 0x2000;
    /// Declared as an `enum` type.
    pub const ACC_ENUM: t = 0x4000;
}

/// Values of access flags for a field.
pub mod field_access_flags {
    #[allow(non_camel_case_types)]
    pub type access_flag = super::u2;
    #[allow(non_camel_case_types)]
    pub type t = access_flag;

    /// Declared `public`; may be accessed from outside its package.
    pub const ACC_PUBLIC: t = 0x0001;
    /// Declared `private`; usable only within the defining class.
    pub const ACC_PRIVATE: t = 0x0002;
    /// Declared `protected`; may be accessed within subclasses.
    pub const ACC_PROTECTED: t = 0x0004;
    /// Declared `static`.
    pub const ACC_STATIC: t = 0x0008;
    /// Declared `final`; no subclasses allowed.
    pub const ACC_FINAL: t = 0x0010;
    /// Declared `volatile`; cannot be cached.
    pub const ACC_VOLATILE: t = 0x0040;
    /// Declared `transient`; not written or read by a persistent object
    /// manager.
    pub const ACC_TRANSIENT: t = 0x0080;
    /// Declared synthetic; not present in the source code.
    pub const ACC_SYNTHETIC: t = 0x1000;
    /// Declared as an element of an `enum`.
    pub const ACC_ENUM: t = 0x4000;
}

/// Values of access flags for a method.
pub mod method_access_flags {
    #[allow(non_camel_case_types)]
    pub type access_flag = super::u2;
    #[allow(non_camel_case_types)]
    pub type t = access_flag;

    /// Declared `public`; may be accessed from outside its package.
    pub const ACC_PUBLIC: t = 0x0001;
    /// Declared `private`; usable only within the defining class.
    pub const ACC_PRIVATE: t = 0x0002;
    /// Declared `protected`; may be accessed within subclasses.
    pub const ACC_PROTECTED: t = 0x0004;
    /// Declared `static`.
    pub const ACC_STATIC: t = 0x0008;
    /// Declared `final`; must not be overriden.
    pub const ACC_FINAL: t = 0x0010;
    /// Declared `synchronized`; invocation is wrapped by a monitor use.
    pub const ACC_SYNCHRONIZED: t = 0x0020;
    /// A bridge method, generated by the compiler.
    pub const ACC_BRIDGE: t = 0x0040;
    /// Declared with variable number of arguments.
    pub const ACC_VARARGS: t = 0x0080;
    /// Declared `native`; implemented in a language other than Java.
    pub const ACC_NATIVE: t = 0x0100;
    /// Declared `abstract`; no implementation is provided.
    pub const ACC_ABSTRACT: t = 0x0400;
    /// Declared `strictfp`; floating-point mode is FP-strict.
    pub const ACC_STRICT: t = 0x0800;
    /// Declared synthetic; not present in the source code.
    pub const ACC_SYNTHETIC: t = 0x1000;
}

/// Values of access flags for parameters.
pub mod parameter_access_flags {
    #[allow(non_camel_case_types)]
    pub type access_flag = super::u2;
    #[allow(non_camel_case_types)]
    pub type t = access_flag;

    /// Indicates that the formal parameter was declared final.
    pub const ACC_FINAL: t = 0x0010;
    /// Indicates that the formal parameter was not explicitly or implicitly
    /// declared in source code, according to the specification of the language
    /// in which the source code was written (JLS §13.1). (The formal parameter
    /// is an implementation artifact of the compiler which produced this class
    /// file.)
    pub const ACC_SYNTHETIC: t = 0x1000;
    /// Indicates that the formal parameter was implicitly declared in source
    /// code, according to the specification of the language in which the source
    /// code was written (JLS §13.1). (The formal parameter is mandated by a
    /// language specification, so all compilers for the language must emit it.)
    pub const ACC_MANDATED: t = 0x8000;
}
