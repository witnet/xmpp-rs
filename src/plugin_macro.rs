#[macro_export]
macro_rules! impl_plugin {
    ($plugin:ty, $proxy:ident, [$(($evt:ty, $pri:expr) => $func:ident),*]) => {
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
                    let new_arc = me.clone();
                    dispatcher.register($pri, move |e: &$evt| {
                        let p = new_arc.as_any().downcast_ref::<$plugin>().unwrap();
                        p . $func(e)
                    });
                )*
            }
        }
    };

    ($plugin:ty, $proxy:ident, [$(($evt:ty, $pri:expr) => $func:ident),*,]) => {
        impl_plugin!($plugin, $proxy, [$(($evt, $pri) => $func),*]);
    };
}
