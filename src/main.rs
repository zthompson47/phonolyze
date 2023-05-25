fn main() -> anyhow::Result<()> {
    pollster::block_on(phonolyze::main())
}
