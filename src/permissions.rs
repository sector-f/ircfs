use std::fmt;

#[derive(Debug, Clone, Copy)]
pub struct Mode {
    pub special: SpecialPerms,
    pub user: Perms,
    pub group: Perms,
    pub other: Perms,
}

impl Mode {
    pub fn new(mode: u32) -> Result<Self, ()> {
        Ok(Mode {
            special: SpecialPerms::new((0o7000 & mode) >> 9)?,
            user: Perms::new((0o700 & mode) >> 6)?,
            group: Perms::new((0o70 & mode) >> 3)?,
            other: Perms::new(0o7 & mode)?,
        })
    }

    pub fn as_int(&self) -> u32 {
          self.special.as_int() * 0o1000
        + self.user.as_int()    * 0o100
        + self.group.as_int()   * 0o10
        + self.other.as_int()
    }
}

impl fmt::Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let ur = if self.user.read { "r" } else { "-" };
        let uw = if self.user.write { "w" } else { "-" };
        let gr = if self.group.read { "r" } else { "-" };
        let gw = if self.group.write { "w" } else { "-" };
        let or = if self.other.read { "r" } else { "-" };
        let ow = if self.other.write { "w" } else { "-" };

        let ux = if self.user.execute && self.special.suid {
            "s"
        } else if self.user.execute {
            "x"
        } else if self.special.suid {
            "S"
        } else {
            "-"
        };
        let gx = if self.group.execute && self.special.sgid {
            "s"
        } else if self.group.execute {
            "x"
        } else if self.special.sgid {
            "S"
        } else {
            "-"
        };
        let ox = if self.other.execute && self.special.sticky {
            "t"
        } else if self.other.execute {
            "x"
        } else if self.special.sticky {
            "T"
        } else {
            "-"
        };

        write!(f, "{}{}{}{}{}{}{}{}{}",
            ur, uw, ux, gr, gw, gx, or, ow, ox
        )
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SpecialPerms {
    suid: bool,
    sgid: bool,
    sticky: bool,
}

impl SpecialPerms {
    pub fn new(int: u32) -> Result<Self, ()> {
        if int > 0b111 { return Err(()) }

        Ok(SpecialPerms {
            suid: int & 0b100 != 0,
            sgid: int & 0b010 != 0,
            sticky: int & 0b001 != 0,
        })
    }

    pub fn as_int(&self) -> u32 {
        let mut value: u32 = 0;

        if self.suid {
            value += 0b100
        }

        if self.sgid {
            value += 0b010
        }

        if self.sticky {
            value += 0b001
        }

        value
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Perms {
    pub read: bool,
    pub write: bool,
    pub execute: bool,
}

impl Perms {
    pub fn new(int: u32) -> Result<Self, ()> {
        if int > 0b111 { return Err(()) }

        Ok(Perms {
            read: int & 0b100 != 0,
            write: int & 0b010 != 0,
            execute: int & 0b001 != 0,
        })
    }

    pub fn as_int(&self) -> u32 {
        let mut value: u32 = 0;

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
