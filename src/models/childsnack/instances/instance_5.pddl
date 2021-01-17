:objects child1 child2 child3 child4 child5 - child
:objects bread1 bread2 bread3 bread4 bread5 - bread-portion
:objects content1 content2 content3 content4 content5 - content-portion
:objects tray1 tray2 tray3 - tray
:objects table1 table2 table3 kitchen - place
:objects sandw1 sandw2 sandw3 sandw4 sandw5 - sandwich

:init at tray1 kitchen
:init at tray2 kitchen
:init at tray3 kitchen
:init at_kitchen_bread bread1
:init at_kitchen_bread bread2
:init at_kitchen_bread bread3
:init at_kitchen_bread bread4
:init at_kitchen_bread bread5
:init at_kitchen_content content1
:init at_kitchen_content content2
:init at_kitchen_content content3
:init at_kitchen_content content4
:init at_kitchen_content content5
:init no_gluten_bread bread2
:init no_gluten_bread bread4
:init no_gluten_content content2
:init no_gluten_content content4
:init allergic_gluten child2
:init allergic_gluten child4
:init not_allergic_gluten child1
:init not_allergic_gluten child3
:init not_allergic_gluten child5
:init waiting child1 table2
:init waiting child2 table1
:init waiting child3 table1
:init waiting child4 table2
:init waiting child5 table3
:init notexist sandw1
:init notexist sandw2
:init notexist sandw3
:init notexist sandw4
:init notexist sandw5

:goal served child1
:goal served child2
:goal served child3
:goal served child4
:goal served child5