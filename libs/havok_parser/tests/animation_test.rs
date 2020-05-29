#[cfg(test)]
mod tests {
    use ffxiv_parser::Pap;
    use havok_parser::{HavokAnimationContainer, HavokBinaryTagFileReader};
    use sqpack_reader::{ExtractedFileProviderWeb, Result, SqPackReaderExtractedFile};

    #[tokio::test]
    async fn pap_test() -> Result<()> {
        let _ = pretty_env_logger::formatted_timed_builder()
            .filter(Some("sqpack_reader"), log::LevelFilter::Debug)
            .try_init();

        let provider = ExtractedFileProviderWeb::new("https://ffxiv-data.dlunch.net/compressed/");
        let pack = SqPackReaderExtractedFile::new(provider);

        let pap = Pap::new(&pack, "chara/human/c1101/animation/a0001/bt_common/resident/idle.pap").await?;
        let hkx = pap.hkx_data();

        let root = HavokBinaryTagFileReader::read(hkx);
        let raw_animation_container = root.find_object_by_type("hkaAnimationContainer");
        let animation_container = HavokAnimationContainer::new(raw_animation_container);

        let havok_animation_binding = &animation_container.bindings[0];
        assert_eq!(havok_animation_binding.transform_track_to_bone_indices[0], 1);

        let havok_animation = &havok_animation_binding.animation;
        assert_eq!(havok_animation.duration(), 0.966666626);

        Ok(())
    }
}