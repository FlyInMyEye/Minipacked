{ lib, rustPlatform, fetchFromGitHub }:

rustPlatform.buildRustPackage rec {
  pname = "minipacked";
  version = "0.1.0";

  src = fetchFromGitHub {
    owner = "FlyInMyEye";
    repo = "Minipacked";
    rev = "v${version}";
    hash = "sha256-0e8hn5cF3PP+EpaOrwPp4lQP9qnPJr884vQ2YX6PLy8=";
  };

  cargoLock = {
    lockFile = src + "/Cargo.lock";
  };

  meta = with lib; {
    description = "Simple tool to pack files and directories into portable (or even encrypted) containers written in rust";
    homepage = "https://github.com/FlyInMyEye/Minipacked";
    license = licenses.mit;
    mainProgram = "minipacked";
    platforms = platforms.linux ++ platforms.darwin;
  };
}
