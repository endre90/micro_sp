:objects general - Lander
:objects colour - Mode
:objects rover0 - Rover
:objects rover0store - Store
:objects waypoint0 waypoint1 - Waypoint
:objects camera0 - Camera
:objects objective0 - Objective

:init visible waypoint1 waypoint0
:init visible waypoint0 waypoint1
:init at_lander general waypoint0
:init channel_free general
:init at rover0 waypoint0
:init available rover0
:init store_of rover0store rover0
:init empty rover0store
:init equipped_for_imaging rover0
:init can_traverse rover0 waypoint1 waypoint0
:init can_traverse rover0 waypoint0 waypoint1
:init on_board camera0 rover0
:init calibration_target camera0 objective0
:init supports camera0 colour
:init visible_from objective0 waypoint0
:init visible_from objective0 waypoint1

:goal communicated_image_data objective0 colour