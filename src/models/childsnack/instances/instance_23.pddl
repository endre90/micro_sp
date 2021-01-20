:objects child1 child2 child3 child4 child5 - child
:objects bread1 bread2 bread3 bread4 bread5 - bread-portion
:objects content1 content2 content3 content4 content5 - content-portion
:objects tray1 tray2 - tray
:objects table1 table2 kitchen - place
:objects sandwich1 sandwich2 sandwich3 sandwich4 sandwich5 - sandwich

:init at tray1 kitchen
:init at tray2 kitchen
:init at_kitchen_bread bread1
:init at_kitchen_content content1
:init at_kitchen_bread bread2
:init at_kitchen_content content2
:init at_kitchen_bread bread3
:init at_kitchen_content content3
:init at_kitchen_bread bread4
:init at_kitchen_content content4
:init at_kitchen_bread bread5
:init at_kitchen_content content5
:init no_gluten_bread bread1
:init no_gluten_content content1
:init no_gluten_bread bread3
:init no_gluten_content content3
:init no_gluten_bread bread5
:init no_gluten_content content5
:init allergic_gluten child1
:init not_allergic_gluten child2
:init allergic_gluten child3
:init not_allergic_gluten child4
:init allergic_gluten child5
:init waiting child1 table1
:init waiting child2 table2
:init waiting child3 table2
:init waiting child4 table2
:init waiting child5 table1
:init notexist sandwich1
:init notexist sandwich2
:init notexist sandwich3
:init notexist sandwich4
:init notexist sandwich5

:goal served child1
:goal served child2
:goal served child3
:goal served child4
:goal served child5