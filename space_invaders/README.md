# Space Invaders emulator

Yet another space invaders emulator.

This isn't the shiniest, nor faster, nor better, but it's mine.

To run it, on the folder of this repository:

```bash
mkdir invaders
mv /some/location/invaders.rom invaders/rom
mv /some/location/{0.wav,1.wav,2.wav,3.wav,4.wav,5.wav,6.wav,7.wav,8.wav} invaders/
cargo run game invaders
```