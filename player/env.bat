@echo off
set RUSTFLAGS=-C target-cpu=sandybridge -C target-feature=+x87,+mmx,+sse,+sse2,+sse3,+ssse3,+sse4.1,+sse4.2,+avx
