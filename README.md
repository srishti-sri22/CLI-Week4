Commands to run in the CLI:

* Compress all files from input folder to output folder
cargo run -- --input ./files --output ./compressed

* Short form
cargo run -- -i ./files -o ./compressed

* Use more threads (8 threads)
cargo run -- -i ./files -o ./compressed -t 8

* Higher compression level (0-9, 9 = best compression)
cargo run -- -i ./files -o ./compressed -l 9

* Combine all options
cargo run -- -i ./files -o ./compressed -t 4 -l 6

* Compress files in current directory
cargo run -- -i . -o ./compressed

* Fast compression (level 1) with 8 threads
cargo run -- -i ./documents -o ./backup -l 1 -t 8

* Maximum compression (level 9) with 2 threads
cargo run -- -i ./videos -o ./compressed_videos -l 9 -t 2

* Compress a single file
cargo run -- -i file.txt -o ./output