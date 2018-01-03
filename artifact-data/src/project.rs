/*  artifact: the requirements tracking tool made for developers
 * Copyright (C) 2017  Garrett Berg <@vitiral, vitiral@gmail.com>
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the Lesser GNU General Public License as published
 * by the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the Lesser GNU General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 * */
//! The major exported type and function for loading artifacts.

use dev_prelude::*;
use lint;
use name::Name;
use implemented::ImplCode;
use settings::ProjectPaths;

#[derive(Debug, Eq, PartialEq)]
pub struct Project {
    pub project_paths: ProjectPaths,

    pub implemented: OrderMap<Name, ImplCode>,
    pub implemented_lints: Vec<lint::Lint>,
}

impl Project {
    /// Recursively sort all the items in the project.
    pub fn sort(&mut self) {
        self.implemented_lints.sort();
        self.implemented_lints.dedup();

        sort_orderset(&mut self.project_paths.code);
        sort_orderset(&mut self.project_paths.artifacts);

        sort_ordermap(&mut self.implemented);
        for (_, code) in self.implemented.iter_mut() {
            sort_ordermap(&mut code.secondary);
        }
    }
}
