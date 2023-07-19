use subenum::subenum;

#[subenum(Edible)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Plant {
    #[subenum(Edible)]
    Basil,
    #[subenum(Edible)]
    Tomato,
    Manzanita,
    Pine,
}

impl std::ops::Deref for Edible {
    type Target = Plant;

    fn deref(&self) -> &Self::Target {
        match self {
            Edible::Basil => &Plant::Basil,
            Edible::Tomato => &Plant::Tomato,
        }
    }
}

fn main() -> Result<(), EdibleConvertError> {
    let plant = Plant::Tomato;

    // We can convert between them.
    let edible = Edible::try_from(plant)?;
    let _plant2 = Plant::from(edible);

    // We can compare them.
    assert_eq!(plant, edible);

    // We derive anything that's derived on the parent, such as clone.
    let edible2 = edible.clone();

    Ok(())
}
