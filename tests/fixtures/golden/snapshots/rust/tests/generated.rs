use linguini_generated::{counted, delivery, EmailInput, Fruit, Size};

#[test]
fn delivery_example_returns_expected_russian_string() {
    assert_eq!(
        delivery(Fruit::Apple, Size::Small, 1),
        "Доставлено маленькое яблоко"
    );
}

#[test]
fn counted_example_returns_expected_plural_strings() {
    assert_eq!(counted(1, Fruit::Apple), "В корзине 1 яблока");
    assert_eq!(counted(5, Fruit::Orange), "В корзине 5 апельсинов");
}

#[test]
fn grouped_message_values_are_available() {
    assert_eq!(EmailInput::LABEL, "Email");
    assert_eq!(EmailInput::ARIA, "Адрес электронной почты");
}
