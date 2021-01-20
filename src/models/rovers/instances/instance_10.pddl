:objects general - Lander
:objects colour high_res low_res - Mode
:objects rover0 - Rover
:objects rover0store - Store
:objects waypoint0 waypoint1 waypoint2 waypoint3 - Waypoint
:objects camera0 - Camera
:objects objective0 objective1 - Objective

:init visible waypoint1 waypoint0
:init visible waypoint0 waypoint1
:init visible waypoint2 waypoint0
:init visible waypoint0 waypoint2
:init visible waypoint2 waypoint1
:init visible waypoint1 waypoint2
:init visible waypoint3 waypoint0
:init visible waypoint0 waypoint3
:init visible waypoint3 waypoint1
:init visible waypoint1 waypoint3
:init visible waypoint3 waypoint2
:init visible waypoint2 waypoint3
:init at_soil_sample waypoint0
:init at_rock_sample waypoint1
:init at_soil_sample waypoint2
:init at_rock_sample waypoint2
:init at_soil_sample waypoint3
:init at_rock_sample waypoint3
:init at_lander general waypoint0
:init channel_free general
:init at rover0 waypoint3
:init available rover0
:init store_of rover0store rover0
:init empty rover0store
:init equipped_for_soil_analysis rover0
:init equipped_for_rock_analysis rover0
:init equipped_for_imaging rover0
:init can_traverse rover0 waypoint3 waypoint0
:init can_traverse rover0 waypoint0 waypoint3
:init can_traverse rover0 waypoint3 waypoint1
:init can_traverse rover0 waypoint1 waypoint3
:init can_traverse rover0 waypoint1 waypoint2
:init can_traverse rover0 waypoint2 waypoint1
:init on_board camera0 rover0
:init calibration_target camera0 objective0
:init calibration_target camera0 objective1
:init supports camera0 colour
:init supports camera0 high_res
:init visible_from objective0 waypoint2
:init visible_from objective1 waypoint3

:goal communicated_image_data objective0 colour
:goal communicated_image_data objective1 colour