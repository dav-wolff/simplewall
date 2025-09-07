{
	description = "Template rust project";
	
	inputs = {
		nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
		flake-utils.url = "github:numtide/flake-utils";
		
		crane = {
			url = "github:ipetkov/crane";
		};
		
		fenix = {
			url = "github:nix-community/fenix";
			inputs.nixpkgs.follows = "nixpkgs";
		};
	};
	
	outputs = { self, nixpkgs, flake-utils, ... } @ inputs: let
		makeCraneLib = pkgs: let
			fenix = inputs.fenix.packages.${pkgs.system};
			## stable
			fenixChannel = fenix.stable;
			fenixToolchain = fenixChannel.withComponents [
				"rustc"
				"cargo"
				"rust-std"
				"rust-docs"
				"clippy"
			];
		in (inputs.crane.mkLib pkgs).overrideToolchain fenixToolchain;
	in {
		overlays = {
			simplewall = final: prev: {
				simplewall = prev.callPackage ./nix/package.nix {
					craneLib = makeCraneLib final;
				};
			};
			
			default = self.overlays.simplewall;
		};
	} // flake-utils.lib.eachDefaultSystem (system:
		let
			pkgs = import nixpkgs {
				inherit system;
				overlays = [self.overlays.default];
			};
			craneLib = makeCraneLib pkgs;
		in {
			packages = {
				inherit (pkgs) simplewall;
				default = self.packages.${system}.simplewall;
			};
			
			devShells.default = craneLib.devShell {
				packages = with pkgs; [
					rust-analyzer
				];
			};
		}
	);
}
