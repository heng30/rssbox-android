{ pkgs ? import <nixpkgs> { } }:
pkgs.mkShell {
  packages = [ pkgs.libGL.dev ];
  inputsFrom = [ pkgs.libGL.dev ];
}
