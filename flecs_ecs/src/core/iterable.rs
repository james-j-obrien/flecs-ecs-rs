use std::marker::PhantomData;

use flecs_ecs_derive::tuples;

use crate::sys::{self, ecs_filter_desc_t, ecs_inout_kind_t, ecs_oper_kind_t};

use super::{
    c_types::{IterT, OperKind, TermT},
    component_registration::ComponentId,
    ecs_field, FilterBuilderImpl, InOutKind, WorldT,
};

pub trait Filterable: Sized + FilterBuilderImpl {
    fn current_term(&mut self) -> &mut TermT;
    fn next_term(&mut self);
}

pub struct ArrayElement {
    pub ptr: *mut u8,
    pub is_ref: bool,
}

pub struct ComponentsData<T: Iterable, const LEN: usize> {
    pub array_components: [*mut u8; LEN],
    pub is_ref_array_components: [bool; LEN],
    pub is_any_array_a_ref: bool,
    _marker: PhantomData<T>,
}

pub trait ComponentPointers<T: Iterable> {
    fn new(iter: &IterT) -> Self;

    fn get_tuple(&mut self, index: usize) -> T::TupleType<'_>;

    fn get_slice(&mut self, count: usize) -> T::TupleSliceType<'_>;
}

impl<T: Iterable, const LEN: usize> ComponentPointers<T> for ComponentsData<T, LEN> {
    fn new(iter: &IterT) -> Self {
        let mut array_components = [std::ptr::null::<u8>() as *mut u8; LEN];
        let mut is_ref_array_components = [false; LEN];

        T::populate_array_ptrs(
            iter,
            &mut array_components[..],
            &mut is_ref_array_components[..],
            &mut 0,
        );

        let is_any_array_a_ref = is_ref_array_components[0];

        Self {
            array_components,
            is_ref_array_components,
            is_any_array_a_ref,
            _marker: PhantomData::<T>,
        }
    }

    fn get_tuple(&mut self, index: usize) -> T::TupleType<'_> {
        if self.is_any_array_a_ref {
            T::create_tuple_with_ref(
                &mut &self.array_components[..],
                &mut &self.is_ref_array_components[..],
                index,
            )
        } else {
            T::create_tuple(&mut &self.array_components[..], index)
        }
    }

    fn get_slice(&mut self, count: usize) -> T::TupleSliceType<'_> {
        if self.is_any_array_a_ref {
            T::create_tuple_slices_with_ref(
                &mut &self.array_components[..],
                &mut &self.is_ref_array_components[..],
                count,
            )
        } else {
            T::create_tuple_slices(&mut &self.array_components[..], count)
        }
    }
}

struct Singleton<T>(T);

pub trait IterableTypeOperation {
    type CastType;
    type ActualType<'w>;
    type SliceType<'w>;
    type OnlyType: ComponentId;

    fn populate_term(term: &mut sys::ecs_term_t);
    fn create_tuple_data<'a>(array_components_data: *mut u8, index: usize) -> Self::ActualType<'a>;
    fn create_tuple_with_ref_data<'a>(
        array_components_data: *mut u8,
        is_ref: bool,
        index: usize,
    ) -> Self::ActualType<'a>;
    fn create_tuple_slice_data<'a>(
        array_components_data: *mut u8,
        count: usize,
    ) -> Self::SliceType<'a>;
    fn create_tuple_slices_with_ref_data<'a>(
        array_components_data: *mut u8,
        is_ref_array_components: bool,
        count: usize,
    ) -> Self::SliceType<'a>;
}

impl<T> IterableTypeOperation for &T
where
    T: ComponentId,
{
    type CastType = *const T;
    type ActualType<'w> = &'w T;
    type SliceType<'w> = &'w [T];
    type OnlyType = T;

    fn populate_term(term: &mut sys::ecs_term_t) {
        term.inout = InOutKind::In as ecs_inout_kind_t;
    }

    fn create_tuple_data<'a>(array_components_data: *mut u8, index: usize) -> Self::ActualType<'a> {
        let data_ptr = array_components_data as Self::CastType;
        unsafe { &*data_ptr.add(index) }
    }

    fn create_tuple_with_ref_data<'a>(
        array_components_data: *mut u8,
        is_ref: bool,
        index: usize,
    ) -> Self::ActualType<'a> {
        let data_ptr = array_components_data as Self::CastType;
        unsafe {
            if is_ref {
                &*data_ptr.add(0)
            } else {
                &*data_ptr.add(index)
            }
        }
    }

    fn create_tuple_slice_data<'a>(
        array_components_data: *mut u8,
        count: usize,
    ) -> Self::SliceType<'a> {
        let data_ptr = array_components_data as Self::CastType;
        unsafe { std::slice::from_raw_parts(data_ptr, count) }
    }

    fn create_tuple_slices_with_ref_data<'a>(
        array_components_data: *mut u8,
        is_ref_array_components: bool,
        count: usize,
    ) -> Self::SliceType<'a> {
        let data_ptr = array_components_data as Self::CastType;
        unsafe {
            if is_ref_array_components {
                std::slice::from_raw_parts(data_ptr, 1)
            } else {
                std::slice::from_raw_parts(data_ptr, count)
            }
        }
    }
}

impl<T> IterableTypeOperation for &mut T
where
    T: ComponentId,
{
    type CastType = *mut T;
    type ActualType<'w> = &'w mut T;
    type SliceType<'w> = &'w mut [T];
    type OnlyType = T;

    fn populate_term(term: &mut sys::ecs_term_t) {
        term.inout = InOutKind::InOut as ecs_inout_kind_t;
    }

    fn create_tuple_data<'a>(array_components_data: *mut u8, index: usize) -> Self::ActualType<'a> {
        let data_ptr = array_components_data as Self::CastType;
        unsafe { &mut *data_ptr.add(index) }
    }

    fn create_tuple_with_ref_data<'a>(
        array_components_data: *mut u8,
        is_ref: bool,
        index: usize,
    ) -> Self::ActualType<'a> {
        let data_ptr = array_components_data as Self::CastType;
        unsafe {
            if is_ref {
                &mut *data_ptr.add(0)
            } else {
                &mut *data_ptr.add(index)
            }
        }
    }

    fn create_tuple_slice_data<'a>(
        array_components_data: *mut u8,
        count: usize,
    ) -> Self::SliceType<'a> {
        let data_ptr = array_components_data as Self::CastType;
        unsafe { std::slice::from_raw_parts_mut(data_ptr, count) }
    }

    fn create_tuple_slices_with_ref_data<'a>(
        array_components_data: *mut u8,
        is_ref_array_components: bool,
        count: usize,
    ) -> Self::SliceType<'a> {
        let data_ptr = array_components_data as Self::CastType;
        unsafe {
            if is_ref_array_components {
                std::slice::from_raw_parts_mut(data_ptr, 1)
            } else {
                std::slice::from_raw_parts_mut(data_ptr, count)
            }
        }
    }
}

impl<T> IterableTypeOperation for Option<&T>
where
    T: ComponentId,
{
    type CastType = *const T;
    type ActualType<'w> = Option<&'w T>;
    type SliceType<'w> = Option<&'w [T]>;
    type OnlyType = T;

    fn populate_term(term: &mut sys::ecs_term_t) {
        term.inout = InOutKind::In as ecs_inout_kind_t;
        term.oper = OperKind::Optional as ecs_oper_kind_t;
    }

    fn create_tuple_data<'a>(array_components_data: *mut u8, index: usize) -> Self::ActualType<'a> {
        let data_ptr = array_components_data as Self::CastType;
        if data_ptr.is_null() {
            None
        } else {
            Some(unsafe { &*data_ptr.add(index) })
        }
    }

    fn create_tuple_with_ref_data<'a>(
        array_components_data: *mut u8,
        is_ref: bool,
        index: usize,
    ) -> Self::ActualType<'a> {
        let data_ptr = array_components_data as Self::CastType;
        if data_ptr.is_null() {
            None
        } else if is_ref {
            Some(unsafe { &*data_ptr.add(0) })
        } else {
            Some(unsafe { &*data_ptr.add(index) })
        }
    }

    fn create_tuple_slice_data<'a>(
        array_components_data: *mut u8,
        count: usize,
    ) -> Self::SliceType<'a> {
        let data_ptr = array_components_data as Self::CastType;
        if data_ptr.is_null() {
            None
        } else {
            Some(unsafe { std::slice::from_raw_parts(data_ptr, count) })
        }
    }

    fn create_tuple_slices_with_ref_data<'a>(
        array_components_data: *mut u8,
        is_ref_array_components: bool,
        count: usize,
    ) -> Self::SliceType<'a> {
        let data_ptr = array_components_data as Self::CastType;
        if data_ptr.is_null() {
            None
        } else if is_ref_array_components {
            Some(unsafe { std::slice::from_raw_parts(data_ptr, 1) })
        } else {
            Some(unsafe { std::slice::from_raw_parts(data_ptr, count) })
        }
    }
}

impl<T> IterableTypeOperation for Option<&mut T>
where
    T: ComponentId,
{
    type CastType = *mut T;
    type ActualType<'w> = Option<&'w mut T>;
    type SliceType<'w> = Option<&'w mut [T]>;
    type OnlyType = T;

    fn populate_term(term: &mut sys::ecs_term_t) {
        term.inout = InOutKind::InOut as ecs_inout_kind_t;
        term.oper = OperKind::Optional as ecs_oper_kind_t;
    }

    fn create_tuple_data<'a>(array_components_data: *mut u8, index: usize) -> Self::ActualType<'a> {
        let data_ptr = array_components_data as Self::CastType;
        if data_ptr.is_null() {
            None
        } else {
            Some(unsafe { &mut *data_ptr.add(index) })
        }
    }

    fn create_tuple_with_ref_data<'a>(
        array_components_data: *mut u8,
        is_ref: bool,
        index: usize,
    ) -> Self::ActualType<'a> {
        let data_ptr = array_components_data as Self::CastType;
        if data_ptr.is_null() {
            None
        } else if is_ref {
            Some(unsafe { &mut *data_ptr.add(0) })
        } else {
            Some(unsafe { &mut *data_ptr.add(index) })
        }
    }

    fn create_tuple_slice_data<'a>(
        array_components_data: *mut u8,
        count: usize,
    ) -> Self::SliceType<'a> {
        let data_ptr = array_components_data as Self::CastType;
        if data_ptr.is_null() {
            None
        } else {
            Some(unsafe { std::slice::from_raw_parts_mut(data_ptr, count) })
        }
    }

    fn create_tuple_slices_with_ref_data<'a>(
        array_components_data: *mut u8,
        is_ref_array_components: bool,
        count: usize,
    ) -> Self::SliceType<'a> {
        let data_ptr = array_components_data as Self::CastType;
        if data_ptr.is_null() {
            None
        } else if is_ref_array_components {
            Some(unsafe { std::slice::from_raw_parts_mut(data_ptr, 1) })
        } else {
            Some(unsafe { std::slice::from_raw_parts_mut(data_ptr, count) })
        }
    }
}

pub trait Iterable: Sized {
    type Pointers: ComponentPointers<Self>;
    type TupleType<'a>;
    type TupleSliceType<'a>;

    fn create_ptrs(iter: &IterT) -> Self::Pointers {
        Self::Pointers::new(iter)
    }

    fn populate(filter: &mut impl Filterable);

    fn register_ids_descriptor(world: *mut WorldT, desc: &mut ecs_filter_desc_t) {
        Self::register_ids_descriptor_at(world, &mut desc.terms[..], &mut 0);
    }

    fn register_ids_descriptor_at(
        world: *mut WorldT,
        terms: &mut [sys::ecs_term_t],
        index: &mut usize,
    );

    fn populate_array_ptrs(
        it: &IterT,
        components: &mut [*mut u8],
        is_ref: &mut [bool],
        index: &mut usize,
    );

    fn create_tuple<'a>(array_components: &mut &'a [*mut u8], index: usize) -> Self::TupleType<'a>;

    fn create_tuple_with_ref<'a>(
        array_components: &mut &'a [*mut u8],
        is_ref_array_components: &mut &'a [bool],
        index: usize,
    ) -> Self::TupleType<'a>;

    fn create_tuple_slices<'a>(
        array_components: &mut &'a [*mut u8],
        count: usize,
    ) -> Self::TupleSliceType<'a>;

    fn create_tuple_slices_with_ref<'a>(
        array_components: &mut &'a [*mut u8],
        is_ref_array_components: &mut &'a [bool],
        count: usize,
    ) -> Self::TupleSliceType<'a>;
}

/////////////////////
// The higher sized tuples are done by a macro towards the bottom of this file.
/////////////////////

#[rustfmt::skip]
impl<A> Iterable for A
where
    A: IterableTypeOperation,
{
    type Pointers = ComponentsData<A, 1>;
    type TupleType<'w> = A::ActualType<'w>;
    type TupleSliceType<'w> = A::SliceType<'w>;

    fn populate(filter: &mut impl Filterable) {
        let world = filter.world_ptr_mut();
        filter.term_with_id(A::OnlyType::get_id(world));
        let term = filter.current_term();
        A::populate_term(term);

    }

    fn register_ids_descriptor_at(
        world: *mut WorldT,
        terms: &mut [sys::ecs_term_t],
        index: &mut usize,
    ) {
        terms[*index].id = A::OnlyType::get_id(world);
        A::populate_term(&mut terms[*index]);
        *index += 1;
    }

    fn populate_array_ptrs(
        it: &IterT,
        components: &mut [*mut u8],
        is_ref: &mut [bool],
        index: &mut usize,
    ) {
        components[*index] =
            unsafe { ecs_field::<A::OnlyType>(it, (*index + 1) as i32) as *mut u8 };
        is_ref[*index] = if !it.sources.is_null() {
            unsafe { *it.sources.add(0) != 0 }
        } else {
            false
        };
        *index += 1;
    }

    fn create_tuple<'a>(array_components: &mut &'a [*mut u8], index: usize) -> Self::TupleType<'a> {
        let data = A::create_tuple_data(array_components[0], index);
        *array_components = &array_components[1..];
        data
    }

    // TODO since it's only one component, we don't need to check if it's a ref array or not, we can just return the first element of the array
    // I think this is the case for all tuples of size 1
    fn create_tuple_with_ref<'a>(
        array_components: &mut &'a [*mut u8],
        is_ref_array_components: &mut &[bool],
        index: usize,
    ) -> Self::TupleType<'a> {
        let data =
            A::create_tuple_with_ref_data(array_components[0], is_ref_array_components[0], index);
        *array_components = &array_components[1..];
        *is_ref_array_components = &is_ref_array_components[1..];
        data
    }

    fn create_tuple_slices<'a>(
        array_components: &mut &'a [*mut u8],
        count: usize,
    ) -> Self::TupleSliceType<'a> {
        let data = A::create_tuple_slice_data(array_components[0], count);
        *array_components = &array_components[1..];
        data
    }

    fn create_tuple_slices_with_ref<'a>(
        array_components: &mut &'a [*mut u8],
        is_ref_array_components: &mut &[bool],
        count: usize,
    ) -> Self::TupleSliceType<'a> {
        let data = A::create_tuple_slices_with_ref_data(
            array_components[0],
            is_ref_array_components[0],
            count,
        );
        *array_components = &array_components[1..];
        *is_ref_array_components = &is_ref_array_components[1..];
        data
    }
}

pub struct Wrapper<T>(T);

pub trait TupleForm<'a, T, U> {
    type Tuple;
    type TupleSlice;
    const IS_OPTION: bool;

    fn return_type_for_tuple(array: *mut U, index: usize) -> Self::Tuple;
    fn return_type_for_tuple_with_ref(array: *mut U, is_ref: bool, index: usize) -> Self::Tuple;
    fn return_type_for_tuple_slices(array: *mut U, count: usize) -> Self::TupleSlice;
    fn return_type_for_tuple_slices_with_ref(
        array: *mut U,
        is_ref: bool,
        count: usize,
    ) -> Self::TupleSlice;
}

impl<'a, T: 'a> TupleForm<'a, T, T> for Wrapper<T> {
    type Tuple = &'a mut T;
    type TupleSlice = &'a mut [T];
    const IS_OPTION: bool = false;

    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    #[inline(always)]
    fn return_type_for_tuple(array: *mut T, index: usize) -> Self::Tuple {
        unsafe { &mut (*array.add(index)) }
    }

    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    #[inline(always)]
    fn return_type_for_tuple_with_ref(array: *mut T, is_ref: bool, index: usize) -> Self::Tuple {
        unsafe {
            if is_ref {
                &mut (*array.add(0))
            } else {
                &mut (*array.add(index))
            }
        }
    }

    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    #[inline(always)]
    fn return_type_for_tuple_slices(array: *mut T, count: usize) -> Self::TupleSlice {
        unsafe { std::slice::from_raw_parts_mut(array, count) }
    }

    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    #[inline(always)]
    fn return_type_for_tuple_slices_with_ref(
        array: *mut T,
        is_ref: bool,
        count: usize,
    ) -> Self::TupleSlice {
        unsafe {
            if is_ref {
                std::slice::from_raw_parts_mut(array, 1)
            } else {
                std::slice::from_raw_parts_mut(array, count)
            }
        }
    }
}

impl<'a, T: 'a> TupleForm<'a, Option<T>, T> for Wrapper<T> {
    type Tuple = Option<&'a mut T>;
    type TupleSlice = Option<&'a mut [T]>;
    const IS_OPTION: bool = true;

    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    #[inline(always)]
    fn return_type_for_tuple(array: *mut T, index: usize) -> Self::Tuple {
        unsafe {
            if array.is_null() {
                None
            } else {
                Some(&mut (*array.add(index)))
            }
        }
    }

    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    #[inline(always)]
    fn return_type_for_tuple_with_ref(array: *mut T, is_ref: bool, index: usize) -> Self::Tuple {
        unsafe {
            if array.is_null() {
                None
            } else if is_ref {
                Some(&mut (*array.add(0)))
            } else {
                Some(&mut (*array.add(index)))
            }
        }
    }

    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    #[inline(always)]
    fn return_type_for_tuple_slices(array: *mut T, count: usize) -> Self::TupleSlice {
        unsafe {
            if array.is_null() {
                None
            } else {
                let slice = std::slice::from_raw_parts_mut(array, count);
                Some(slice)
            }
        }
    }

    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    #[inline(always)]
    fn return_type_for_tuple_slices_with_ref(
        array: *mut T,
        is_ref: bool,
        count: usize,
    ) -> Self::TupleSlice {
        unsafe {
            if array.is_null() {
                None
            } else if is_ref {
                let slice = std::slice::from_raw_parts_mut(array, 1);
                Some(slice)
            } else {
                let slice = std::slice::from_raw_parts_mut(array, count);
                Some(slice)
            }
        }
    }
}

macro_rules! tuple_count {
    () => { 0 };
    ($head:ident) => { 1 };
    ($head:ident, $($tail:ident),*) => { 1 + tuple_count!($($tail),*) };
}

macro_rules! impl_iterable {
    ($($t:ident),*) => {
        impl<$($t: IterableTypeOperation),*> Iterable for ($($t,)*) {
            type TupleType<'w> = ($(
                $t::ActualType<'w>,
            )*);

            type TupleSliceType<'w> = ($(
                $t::SliceType<'w>,
            )*);
            type Pointers = ComponentsData<Self, { tuple_count!($($t),*) }>;


            fn populate(filter: &mut impl Filterable) {
                let _world = filter.world_ptr_mut();
                $(
                    filter.term_with_id($t::OnlyType::get_id(_world));
                    let term = filter.current_term();
                    $t::populate_term(term);

                )*
            }

            #[allow(unused)]
            fn register_ids_descriptor_at(world: *mut WorldT, terms: &mut [sys::ecs_term_t], index: &mut usize) {
                $( $t::register_ids_descriptor_at(world, terms, index); )*
            }

            fn populate_array_ptrs(
                _it: &IterT,
                _components: &mut [*mut u8],
                _is_ref: &mut [bool],
                _index: &mut usize,
            ) {
                $( $t::populate_array_ptrs(_it, _components, _is_ref, _index); )*
            }

            #[allow(unused)]
            fn create_tuple<'a>(array_components: &mut &'a [*mut u8], index: usize) -> Self::TupleType<'a> {
                ($( $t::create_tuple(array_components, index), )*)
            }

            #[allow(unused)]
            fn create_tuple_with_ref<'a>(array_components: &mut &'a [*mut u8], is_ref_array_components: &mut &'a [bool], index: usize) -> Self::TupleType<'a> {
                ($( $t::create_tuple_with_ref(array_components, is_ref_array_components, index), )*)
            }

            #[allow(unused)]
            fn create_tuple_slices<'a>(
                array_components: &mut &'a [*mut u8],
                count: usize,
            ) -> Self::TupleSliceType<'a> {
                ($( $t::create_tuple_slices(array_components, count), )*)
            }

            #[allow(unused)]
            fn create_tuple_slices_with_ref<'a>(
                array_components: &mut &'a [*mut u8],
                is_ref_array_components: &mut &'a [bool],
                count: usize,
            ) -> Self::TupleSliceType<'a> {
                ($( $t::create_tuple_slices_with_ref(array_components, is_ref_array_components, count), )*)
            }
        }
    }
}

tuples!(impl_iterable, 0, 12);
