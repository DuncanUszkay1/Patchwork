macro_rules! define_interface {
    ($name:ident,
            $( ( $op:ident, $op_method:ident, [ $( $field_name:ident: $field_type:ty ),* ] ) ),*
    ) => {
        pub trait $name {
            $( fn $op_method(&self, $( $field_name: $field_type ),*); )*
        }

        impl $name for Sender<Operations> {
            $(
                fn $op_method(&self, $( $field_name: $field_type ),*) {
                    self.send(Operations::$op($op { $( $field_name ),* }))
                        .unwrap();
                }
            )*
        }

        pub enum Operations {
            $( $op($op), )*
        }

        $(
            #[derive(Debug)]
            pub struct $op {
                $( pub $field_name: $field_type, )*
            }
        )*
    }
}
