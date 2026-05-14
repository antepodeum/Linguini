#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fruit {
    Apple,
    Pear,
    Orange,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Size {
    Small,
    Big,
}

pub struct FruitForm {
    gender: Gender,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Gender {
    Male,
    Female,
    Neuter,
}

impl Fruit {
    fn form(self) -> FruitForm {
        match self {
            Self::Apple => FruitForm {
                gender: Gender::Neuter,
            },
            Self::Pear => FruitForm {
                gender: Gender::Female,
            },
            Self::Orange => FruitForm {
                gender: Gender::Male,
            },
        }
    }

    fn nom(self, count: u64) -> &'static str {
        match (self, plural_ru(count)) {
            (Self::Apple, "one") => "яблоко",
            (Self::Apple, "few") => "яблока",
            (Self::Apple, _) => "яблок",
            (Self::Pear, "one") => "груша",
            (Self::Pear, "few") => "груши",
            (Self::Pear, _) => "груш",
            (Self::Orange, "one") => "апельсин",
            (Self::Orange, "few") => "апельсина",
            (Self::Orange, _) => "апельсинов",
        }
    }

}

fn delivered(count: u64, gender: Gender) -> &'static str {
    match plural_ru(count) {
        "one" => match gender {
            Gender::Male => "Доставлен",
            Gender::Female => "Доставлена",
            Gender::Neuter => "Доставлено",
        },
        _ => "Доставлены",
    }
}

fn size_adj(size: Size, count: u64, gender: Gender) -> &'static str {
    match size {
        Size::Small => match plural_ru(count) {
            "one" => match gender {
                Gender::Male => "маленький",
                Gender::Female => "маленькая",
                Gender::Neuter => "маленькое",
            },
            _ => "маленьких",
        },
        Size::Big => match plural_ru(count) {
            "one" => match gender {
                Gender::Male => "большой",
                Gender::Female => "большая",
                Gender::Neuter => "большое",
            },
            _ => "больших",
        },
    }
}

fn plural_ru(value: u64) -> &'static str {
    let mod10 = value % 10;
    let mod100 = value % 100;

    if mod10 == 1 && mod100 != 11 {
        "one"
    } else if (2..=4).contains(&mod10) && !(12..=14).contains(&mod100) {
        "few"
    } else if mod10 == 0 || (5..=9).contains(&mod10) || (11..=14).contains(&mod100) {
        "many"
    } else {
        "other"
    }
}

pub fn delivery(fruit: Fruit, size: Size, count: u64) -> String {
    let form = fruit.form();
    format!(
        "{} {} {}",
        delivered(count, form.gender),
        size_adj(size, count, form.gender),
        fruit.nom(count)
    )
}

pub fn counted(count: u64, fruit: Fruit) -> String {
    format!("В корзине {} {}", count, fruit.nom(count))
}

pub struct EmailInput;

impl EmailInput {
    pub const LABEL: &'static str = "Email";
    pub const PLACEHOLDER: &'static str = "name@example.com";
    pub const ARIA: &'static str = "Адрес электронной почты";
}
