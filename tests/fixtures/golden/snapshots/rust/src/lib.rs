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

}

fn delivered(gender: Gender, plural: &str) -> &'static str {
    match (gender, plural) {
        (Gender::Male, "one") => "Доставлен",
        (Gender::Female, "one") => "Доставлена",
        (Gender::Neuter, "one") => "Доставлено",
        _ => "Доставлено",
    }
}

fn size_label(size: Size, gender: Gender, plural: &str) -> &'static str {
    match (size, gender, plural) {
        (Size::Small, Gender::Male, "one") => "маленький",
        (Size::Small, Gender::Female, "one") => "маленькая",
        (Size::Small, Gender::Neuter, "one") => "маленькое",
        (Size::Small, Gender::Female, "few") => "маленькие",
        (Size::Small, _, "few" | "many") => "маленьких",
        (Size::Big, Gender::Male, "one") => "большой",
        (Size::Big, Gender::Female, "one") => "большая",
        (Size::Big, Gender::Neuter, "one") => "большое",
        (Size::Big, Gender::Female, "few") => "большие",
        (Size::Big, _, "few" | "many") => "больших",
        _ => "обычные",
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
    let plural = plural_ru(count);
    format!(
        "{} {} {}",
        delivered(form.gender, plural),
        size_label(size, form.gender, plural),
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
