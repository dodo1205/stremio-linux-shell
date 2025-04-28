#[macro_export]
macro_rules! cef_impl {
    (
        prefix = $prefix:literal,
        name = $name:ident,
        sys_type = $sys:ty,
        { $($body:tt)* }
    ) => {
        paste::paste! {
            use cef::{rc::*, *};

            pub struct [<$prefix $name>] {
                object: *mut RcImpl<$sys, Self>,
            }

            impl [<$prefix $name>] {
                #[allow(clippy::new_ret_no_self)]
                pub fn new() -> $name {
                    $name::new(Self {
                        object: std::ptr::null_mut(),
                    })
                }
            }

            impl [<"Wrap" $name>] for [<$prefix $name>] {
                fn wrap_rc(&mut self, object: *mut RcImpl<$sys, Self>) {
                    self.object = object;
                }
            }

            impl Clone for [<$prefix $name>] {
                fn clone(&self) -> Self {
                    let object = unsafe {
                        let rc_impl = &mut *self.object;
                        rc_impl.interface.add_ref();
                        rc_impl
                    };

                    Self { object }
                }
            }

            impl Rc for [<$prefix $name>] {
                fn as_base(&self) -> &cef_dll_sys::cef_base_ref_counted_t {
                    unsafe {
                        let base = &*self.object;
                        std::mem::transmute(&base.cef_object)
                    }
                }
            }

            impl [<"Impl" $name>] for [<$prefix $name>] {
                fn get_raw(&self) -> *mut $sys {
                    self.object.cast()
                }

                $($body)*
            }
        }
    };
}
