// Employee management system using enum and struct
enum Department {
    ENGINEERING,
    SALES,
    MARKETING,
    HR
};

enum EmployeeLevel {
    JUNIOR,
    SENIOR,
    MANAGER
};

struct Employee {
    int id;
    enum Department dept;
    enum EmployeeLevel level;
    int salary_base;
};

int calculate_salary(struct Employee *emp) {
    int salary;
    int dept_bonus;
    int level_bonus;
    
    salary = emp->salary_base;
    
    // Department bonus
    dept_bonus = 0;
    if (emp->dept == ENGINEERING) {
        dept_bonus = 10;
    }
    if (emp->dept == SALES) {
        dept_bonus = 15;
    }
    if (emp->dept == MARKETING) {
        dept_bonus = 8;
    }
    if (emp->dept == HR) {
        dept_bonus = 5;
    }
    
    // Level bonus
    level_bonus = 0;
    if (emp->level == JUNIOR) {
        level_bonus = 0;
    }
    if (emp->level == SENIOR) {
        level_bonus = 20;
    }
    if (emp->level == MANAGER) {
        level_bonus = 50;
    }
    
    return salary + dept_bonus + level_bonus;
}

int main() {
    struct Employee emp[3];
    int total_payroll;
    
    // Employee 1: Junior Engineer
    // 50 + 10 + 0 = 60
    emp[0].id = 1;
    emp[0].dept = ENGINEERING;
    emp[0].level = JUNIOR;
    emp[0].salary_base = 50;
    
    // Employee 2: Senior Sales
    // 50 + 15 + 20 = 85
    emp[1].id = 2;
    emp[1].dept = SALES;
    emp[1].level = SENIOR;
    emp[1].salary_base = 50;
    
    // Employee 3: HR Manager
    // 50 + 5 + 50 = 105
    emp[2].id = 3;
    emp[2].dept = HR;
    emp[2].level = MANAGER;
    emp[2].salary_base = 50;
    
    // Calculate total payroll
    total_payroll = 0;
    int i;
    for (i = 0; i < 3; i++) {
        total_payroll += calculate_salary(&emp[i]);
    }
    return total_payroll; // 60 + 85 + 105 = 250
}
