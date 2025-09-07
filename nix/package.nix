{ lib
, craneLib
}:

let
	src = with lib; cleanSourceWith {
		src = craneLib.path ../.;
		filter = craneLib.filterCargoSources;
	};
	
	nameVersion = craneLib.crateNameFromCargoToml { inherit src; };
	pname = nameVersion.pname;
	version = nameVersion.version;
	
	commonArgs = {
		inherit pname version src;
	};
	
	cargoArtifacts = craneLib.buildDepsOnly commonArgs;
in craneLib.buildPackage (commonArgs // {
	inherit cargoArtifacts;
})
