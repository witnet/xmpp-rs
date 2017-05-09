#[macro_export]
macro_rules! impl_plugin {
    ($plugin:ty, $proxy:ident, [$($evt:ty => $pri:expr),*]) => {
        impl $crate::plugin::Plugin for $plugin {
            fn get_proxy(&mut self) -> &mut $crate::plugin::PluginProxy {
                &mut self.$proxy
            }
        }

        #[allow(unused_variables)]
        impl $crate::plugin::PluginInit for $plugin {
            fn init( dispatcher: &mut $crate::event::Dispatcher
                   , me: ::std::sync::Arc<Box<$crate::plugin::Plugin>>) {
                $(
                    dispatcher.register($pri, unsafe {
                        $crate::event::EventProxy::new::<$plugin>(me.clone())
                    });
                )*
            }
        }
    };

    ($plugin:ty, $proxy:ident, [$($evt:ty => $pri:expr),*,]) => {
        impl_plugin!($plugin, $proxy, [$($evt => $pri),*]);
    };
}
