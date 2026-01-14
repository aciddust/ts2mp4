# deployment guide

## Automatic

```bash
cargo install cargo-release

cargo release patch
cargo release minor
cargo release major
cargo release x.y.z
```

## Manual

### Update [Cargo.toml](/Cargo.toml)

```toml
version = "x.y.z"
```

### Commit changes

```bash
git add Cargo.toml
git commit -m "chore: bump version to x.y.z"
```

### Tag and push

```bash
git tag vx.y.z
git push origin vx.y.z
```
