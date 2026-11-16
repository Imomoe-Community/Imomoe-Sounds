// Copyright (C) 2026 Mai Ooizumi
// SPDX-License-Identifier: AGPL-3.0-or-later

// Test-only host values follow Plugify's three-word string/vector layouts.
use std::{mem::MaybeUninit, slice};

pub(crate) fn init_plugify_values() {
    plugify::string::init_construct_string(construct_string as *const () as usize);
    plugify::string::init_destroy_string(destroy_string as *const () as usize);
    plugify::string::init_get_string_data(string_data as *const () as usize);
    plugify::string::init_get_string_length(string_len as *const () as usize);
    plugify::vector::init_construct_vector_int32(construct_ints as *const () as usize);
    plugify::vector::init_destroy_vector_int32(destroy_ints as *const () as usize);
    plugify::vector::init_get_vector_size_int32(ints_len as *const () as usize);
    plugify::vector::init_get_vector_data_int32(ints_data as *const () as usize);
}

unsafe extern "C" fn construct_string(data: *const u8, size: usize) -> plugify::Str {
    let mut bytes = if size == 0 {
        Vec::new()
    } else {
        unsafe { slice::from_raw_parts(data, size) }.to_vec()
    };
    let mut value = MaybeUninit::<plugify::Str>::uninit();
    unsafe {
        let fields = value.as_mut_ptr().cast::<usize>();
        fields.write(bytes.as_mut_ptr() as usize);
        fields.add(1).write(bytes.len());
        fields.add(2).write(bytes.capacity());
        std::mem::forget(bytes);
        value.assume_init()
    }
}

unsafe extern "C" fn destroy_string(value: *mut plugify::Str) {
    unsafe {
        let fields = value.cast::<usize>();
        if *fields.add(2) != 0 {
            drop(Vec::from_raw_parts(
                *fields as *mut u8,
                *fields.add(1),
                *fields.add(2),
            ));
        }
    }
}
unsafe extern "C" fn string_data(value: *const plugify::Str) -> *mut u8 {
    unsafe { *value.cast::<usize>() as *mut u8 }
}
unsafe extern "C" fn string_len(value: *const plugify::Str) -> usize {
    unsafe { *value.cast::<usize>().add(1) }
}

unsafe extern "C" fn construct_ints(data: *const i32, size: usize) -> plugify::Arr<i32> {
    let mut values = if size == 0 {
        Vec::new()
    } else {
        unsafe { slice::from_raw_parts(data, size) }.to_vec()
    };
    let mut result = MaybeUninit::<plugify::Arr<i32>>::uninit();
    unsafe {
        let fields = result.as_mut_ptr().cast::<usize>();
        let begin = values.as_mut_ptr() as usize;
        fields.write(begin);
        fields
            .add(1)
            .write(begin + values.len() * std::mem::size_of::<i32>());
        fields
            .add(2)
            .write(begin + values.capacity() * std::mem::size_of::<i32>());
        std::mem::forget(values);
        result.assume_init()
    }
}
unsafe extern "C" fn destroy_ints(value: *mut plugify::Arr<i32>) {
    unsafe {
        let fields = value.cast::<usize>();
        let begin = *fields;
        if begin != 0 {
            let size = std::mem::size_of::<i32>();
            drop(Vec::from_raw_parts(
                begin as *mut i32,
                (*fields.add(1) - begin) / size,
                (*fields.add(2) - begin) / size,
            ));
        }
    }
}
unsafe extern "C" fn ints_data(value: *const plugify::Arr<i32>) -> *mut i32 {
    unsafe { *value.cast::<usize>() as *mut i32 }
}
unsafe extern "C" fn ints_len(value: *const plugify::Arr<i32>) -> usize {
    unsafe {
        let fields = value.cast::<usize>();
        (*fields.add(1) - *fields) / std::mem::size_of::<i32>()
    }
}
