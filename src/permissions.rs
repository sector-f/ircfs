#[derive(Debug)]
pub struct Mode {
    pub user: Perms,
    pub group: Perms,
    pub other: Perms,
}

impl Mode {
    // Converts octal number into a Mode
    pub fn new(mode: u16) -> Result<Self, ()> {
        if mode > 0o777 { return Err(()) }

        Ok(Mode {
            user: Perms::new((0o700 & mode) >> 6).unwrap(),
            group: Perms::new((0o70 & mode) >> 3).unwrap(),
            other: Perms::new(0o7 & mode).unwrap(),
        })
    }

    pub fn from_perms(user: Perms, group: Perms, other: Perms) -> Self {
        Mode {
            user: user,
            group: group,
            other: other,
        }
    }

    pub fn as_int(&self) -> u16 {
        self.user.as_int() * 0o100
        + self.group.as_int() * 0o10
        + self.other.as_int()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Perms {
    pub read: bool,
    pub write: bool,
    pub execute: bool,
}

impl Perms {
    pub fn new(int: u16) -> Result<Self, ()> {
        if int > 0b111 { return Err(()) }

        Ok(Perms {
            read: int & 0b100 != 0,
            write: int & 0b010 != 0,
            execute: int & 0b001 != 0,
        })
    }

    pub fn from_bools(r: bool, w: bool, x: bool) -> Self {
        Perms {
            read: r,
            write: w,
            execute: x,
        }
    }

    pub fn as_int(&self) -> u16 {
        let mut value: u16 = 0;

        if self.read {
            value += 0b100
        }

        if self.write {
            value += 0b010
        }

        if self.execute {
            value += 0b001
        }

        value
    }
}

fn main() {
    let mode = Mode::new(0o755).unwrap();
    println!("{:#?}", mode);
}
