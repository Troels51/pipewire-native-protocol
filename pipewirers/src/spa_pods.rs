use zerocopy::AsBytes;

#[derive(AsBytes)]
#[repr(C, align(8))]
pub(crate) struct Header {
    size: u32,
    spa_type: u32,
}

#[repr(C)]
pub(crate) struct None {
    header: Header,
}
impl None {
    pub(crate) fn new() -> Self {
        Self {
            header: Header {
                size: 0,
                spa_type: 1,
            },
        }
    }
}

#[derive(AsBytes)]
#[repr(C, align(8))]
pub(crate) struct Bool {
    header: Header,
    value: i32,
    _pad: u32,
}
impl Bool {
    pub(crate) fn new(value: bool) -> Self {
        Self {
            header: Header {
                size: 4,
                spa_type: 2,
            },
            value: if value { 1 } else { 0 },
            _pad: 0,
        }
    }
}

#[derive(AsBytes)]
#[repr(C, align(8))]
pub(crate) struct Id {
    header: Header,
    id: u32,
    _pad: u32,
}
impl Id {
    pub(crate) fn new(id: u32) -> Self {
        Self {
            header: Header {
                size: 4,
                spa_type: 3,
            },
            id: id,
            _pad: 0,
        }
    }
}

#[derive(AsBytes)]
#[repr(C, align(8))]
pub(crate) struct Int {
    header: Header,
    value: i32,
    _pad: u32,
}
impl Int {
    pub(crate) fn new(value: i32) -> Self {
        Self {
            header: Header {
                size: 4,
                spa_type: 4,
            },
            value: value,
            _pad: 0,
        }
    }
}

#[derive(AsBytes)]
#[repr(C, align(8))]
pub(crate) struct Struct<T> {
    header: Header,
    value: i32,
    _pad: u32,
}
impl Struct {
    pub(crate) fn new(value: i32) -> Self {
        Self {
            header: Header {
                size: SOME_SIZE,
                spa_type: 14,
            },
            value: value,
            _pad: 0,
        }
    }
}
