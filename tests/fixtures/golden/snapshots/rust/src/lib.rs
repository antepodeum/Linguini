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
            (Self::Apple, "many") => "яблок",
            (Self::Pear, "one") => "груша",
            (Self::Pear, "few") => "груши",
            (Self::Pear, "many") => "груш",
            (Self::Orange, "one") => "апельсин",
            (Self::Orange, "few") => "апельсина",
            (Self::Orange, "many") => "апельсинов",
            (Self::Apple, _) => "яблока",
            (Self::Pear, _) => "груши",
            (Self::Orange, _) => "апельсина",
        }
    }

    fn gen(self, count: u64) -> &'static str {
        match (self, plural_ru(count)) {
            (Self::Apple, "one") => "яблока",
            (Self::Apple, "few" | "many") => "яблок",
            (Self::Pear, "one") => "груши",
            (Self::Pear, "few" | "many") => "груш",
            (Self::Orange, "one") => "апельсина",
            (Self::Orange, "few" | "many") => "апельсинов",
            (Self::Apple, _) => "яблока",
            (Self::Pear, _) => "груши",
            (Self::Orange, _) => "апельсина",
        }
    }
}

impl Size {
    fn select(self, gender: Gender) -> &'static str {
        match (self, gender) {
            (Self::Small, Gender::Male) => "маленький",
            (Self::Small, Gender::Female) => "маленькая",
            (Self::Small, Gender::Neuter) => "маленькое",
            (Self::Big, Gender::Male) => "большой",
            (Self::Big, Gender::Female) => "большая",
            (Self::Big, Gender::Neuter) => "большое",
        }
    }
}

fn delivered(gender: Gender) -> &'static str {
    match gender {
        Gender::Male => "Доставлен",
        Gender::Female => "Доставлена",
        Gender::Neuter => "Доставлено",
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
        delivered(form.gender),
        size.select(form.gender),
        fruit.nom(count)
    )
}

pub fn counted(count: u64, fruit: Fruit) -> String {
    format!("В корзине {} {}", count, fruit.gen(count))
}

pub struct EmailInput;

impl EmailInput {
    pub const LABEL: &'static str = "Email";
    pub const PLACEHOLDER: &'static str = "name@example.com";
    pub const ARIA: &'static str = "Адрес электронной почты";
}
