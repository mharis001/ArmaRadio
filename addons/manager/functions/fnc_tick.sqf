#include "script_component.hpp"

private _player = call CBA_fnc_currentUnit;

private _d = eyeDirection _player;
private _u = vectorUp _player;
EXT callExtension ["orientation", [_d#0, _d#1, _d#2, _u#0, _u#1, _u#2]];

{
	private _source = GVAR(active) getVariable [_x, objNull];
	if (alive _source) then {
		private _pos = getPosASL _source;
		private _data = [_x,0,0,0];
		if !(_source isEqualTo vehicle _player) then {
			private _ppos = eyePos _player;
			_data = [
				_x,
				(_pos#0 - _ppos#0) toFixed 2,
				(_pos#1 - _ppos#1) toFixed 2,
				(_pos#2 - _ppos#2) toFixed 2
			];
		};
		EXT callExtension ["pos", _data];
	} else {
		if !(_souce isEqualTo objNull) then {
			[QGVAR(stop), [_x]] call CBA_fnc_localEvent;
		};
	};
} forEach allVariables GVAR(active);
