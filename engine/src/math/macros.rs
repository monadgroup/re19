macro_rules! flatten_op_components (
    ($vec_name:ident, $self_ident:ident, $rhs_ident:ident, $func_name:ident, ($($component:ident),*)) => (
        $vec_name {
            $(
                $component: $self_ident.$component.$func_name($rhs_ident.$component),
            )*
        }
    );
);
macro_rules! flatten_op_rhs_const (
    ($vec_name:ident, $self_ident:ident, $rhs_ident:ident, $func_name:ident, ($($component:ident),*)) => (
        $vec_name {
            $(
                $component: $self_ident.$component.$func_name($rhs_ident),
            )*
        }
    );
);
macro_rules! flatten_op_lhs_const (
    ($vec_name:ident, $self_ident:ident, $rhs_ident:ident, $func_name:ident, ($($component:ident),*)) => (
        $vec_name {
            $(
                $component: $self_ident.$func_name($rhs_ident.$component),
            )*
        }
    )
);

macro_rules! define_vec_operators (
    ($vec_name:ident => $components:tt => $($trait_name:ident = $func_name:ident),*) => (
        $(
            impl ops::$trait_name<$vec_name> for $vec_name {
                type Output = $vec_name;

                fn $func_name(self, rhs: $vec_name) -> Self::Output {
                    flatten_op_components!($vec_name, self, rhs, $func_name, $components)
                }
            }

            impl ops::$trait_name<f32> for $vec_name {
                type Output = $vec_name;

                fn $func_name(self, rhs: f32) -> Self::Output {
                    flatten_op_rhs_const!($vec_name, self, rhs, $func_name, $components)
                }
            }

            impl ops::$trait_name<$vec_name> for f32 {
                type Output = $vec_name;

                fn $func_name(self, rhs: $vec_name) -> Self::Output {
                    flatten_op_lhs_const!($vec_name, self, rhs, $func_name, $components)
                }
            }
        )*
    )
);

macro_rules! flatten_assign_components(
    ($self_ident:ident, $rhs_ident:ident, $func_name:ident, ($($component:ident),*)) => (
        {
            $($self_ident.$component.$func_name($rhs_ident.$component); )*
        }
    );
);

macro_rules! define_vec_assignments(
    ($vec_name:ident => $components:tt => $($trait_name:ident = $func_name:ident),*) => (
        $(
            impl ops::$trait_name<$vec_name> for $vec_name {
                fn $func_name(&mut self, rhs: $vec_name) {
                    flatten_assign_components!(self, rhs, $func_name, $components)
                }
            }
        )*
    )
);

macro_rules! define_vec_common_operators(
    ($vec_name:ident => $components:tt) => (
        define_vec_operators!(
            $vec_name => $components =>
                Add = add,
                Div = div,
                Mul = mul,
                Rem = rem,
                Sub = sub
        );
    )
);

macro_rules! define_vec_common_assignments(
    ($vec_name:ident => $components:tt) => (
        define_vec_assignments!(
            $vec_name => $components =>
                AddAssign = add_assign,
                DivAssign = div_assign,
                MulAssign = mul_assign,
                RemAssign = rem_assign,
                SubAssign = sub_assign
        );
    )
);

macro_rules! define_vec(
    ($vec_name:ident => $components:tt) => (
        define_vec_common_operators!($vec_name => $components);
        define_vec_common_assignments!($vec_name => $components);
    )
);
