:objects general - Lander
:objects colour high_res low_res - Mode
:objects rover0 rover1 - Rover
:objects rover0store rover1store - Store
:objects waypoint0 waypoint1 waypoint2 waypoint3 waypoint4 waypoint5 - Waypoint
:objects camera0 camera1 camera2 - Camera
:objects objective0 objective1 - Objective

:init visible waypoint0 waypoint3
:init visible waypoint3 waypoint0
:init visible waypoint1 waypoint0
:init visible waypoint0 waypoint1
:init visible waypoint1 waypoint4
:init visible waypoint4 waypoint1
:init visible waypoint2 waypoint0
:init visible waypoint0 waypoint2
:init visible waypoint2 waypoint3
:init visible waypoint3 waypoint2
:init visible waypoint3 waypoint1
:init visible waypoint1 waypoint3
:init visible waypoint3 waypoint5
:init visible waypoint5 waypoint3
:init visible waypoint4 waypoint3
:init visible waypoint3 waypoint4
:init visible waypoint5 waypoint0
:init visible waypoint0 waypoint5
:init visible waypoint5 waypoint2
:init visible waypoint2 waypoint5
:init visible waypoint5 waypoint4
:init visible waypoint4 waypoint5
:init at_rock_sample waypoint0
:init at_soil_sample waypoint1
:init at_soil_sample waypoint2
:init at_rock_sample waypoint2
:init at_soil_sample waypoint3
:init at_rock_sample waypoint3
:init at_soil_sample waypoint4
:init at_soil_sample waypoint5
:init at_rock_sample waypoint5
:init at_lander general waypoint3
:init channel_free general
:init at rover0 waypoint1
:init available rover0
:init store_of rover0store rover0
:init empty rover0store
:init equipped_for_rock_analysis rover0
:init equipped_for_imaging rover0
:init can_traverse rover0 waypoint1 waypoint0
:init can_traverse rover0 waypoint0 waypoint1
:init can_traverse rover0 waypoint1 waypoint4
:init can_traverse rover0 waypoint4 waypoint1
:init can_traverse rover0 waypoint0 waypoint2
:init can_traverse rover0 waypoint2 waypoint0
:init can_traverse rover0 waypoint0 waypoint3
:init can_traverse rover0 waypoint3 waypoint0
:init can_traverse rover0 waypoint0 waypoint5
:init can_traverse rover0 waypoint5 waypoint0
:init at rover1 waypoint4
:init available rover1
:init store_of rover1store rover1
:init empty rover1store
:init equipped_for_soil_analysis rover1
:init equipped_for_imaging rover1
:init can_traverse rover1 waypoint4 waypoint1
:init can_traverse rover1 waypoint1 waypoint4
:init can_traverse rover1 waypoint4 waypoint3
:init can_traverse rover1 waypoint3 waypoint4
:init can_traverse rover1 waypoint4 waypoint5
:init can_traverse rover1 waypoint5 waypoint4
:init can_traverse rover1 waypoint1 waypoint0
:init can_traverse rover1 waypoint0 waypoint1
:init can_traverse rover1 waypoint5 waypoint2
:init can_traverse rover1 waypoint2 waypoint5
:init on_board camera0 rover0
:init calibration_target camera0 objective0
:init supports camera0 colour
:init supports camera0 low_res
:init on_board camera1 rover0
:init calibration_target camera1 objective1
:init supports camera1 colour
:init supports camera1 low_res
:init on_board camera2 rover1
:init calibration_target camera2 objective0
:init supports camera2 colour
:init supports camera2 high_res
:init visible_from objective0 waypoint0
:init visible_from objective0 waypoint1
:init visible_from objective0 waypoint2
:init visible_from objective0 waypoint3
:init visible_from objective0 waypoint4
:init visible_from objective0 waypoint5
:init visible_from objective1 waypoint0
:init visible_from objective1 waypoint1

:goal communicated_soil_data waypoint5
:goal communicated_soil_data waypoint1
:goal communicated_soil_data waypoint4
:goal communicated_soil_data waypoint2
:goal communicated_rock_data waypoint0
:goal communicated_rock_data waypoint2
:goal communicated_rock_data waypoint3
:goal communicated_image_data objective0 colour
:goal communicated_image_data objective1 low_res
:goal communicated_image_data objective0 low_res