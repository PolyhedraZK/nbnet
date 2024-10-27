use bip39::{Language, Mnemonic, MnemonicType};

pub fn create_mnemonic_words() -> String {
    Mnemonic::new(MnemonicType::Words24, Language::English)
        .phrase()
        .to_owned()
}
