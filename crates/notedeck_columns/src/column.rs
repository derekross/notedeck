use crate::{
    actionbar::TimelineOpenResult,
    route::{Route, Router},
    timeline::{Timeline, TimelineCache, TimelineKind},
};
use enostr::RelayPool;
use nostrdb::{Ndb, Transaction};
use notedeck::NoteCache;
use std::iter::Iterator;
use tracing::warn;

#[derive(Clone)]
pub struct Column {
    router: Router<Route>,
}

impl Column {
    pub fn new(routes: Vec<Route>) -> Self {
        let router = Router::new(routes);
        Column { router }
    }

    pub fn router(&self) -> &Router<Route> {
        &self.router
    }

    pub fn router_mut(&mut self) -> &mut Router<Route> {
        &mut self.router
    }
}

#[derive(Default)]
pub struct Columns {
    /// Columns are simply routers into settings, timelines, etc
    columns: Vec<Column>,

    /// The selected column for key navigation
    selected: i32,
}

impl Columns {
    pub fn new() -> Self {
        Columns::default()
    }

    pub fn add_new_timeline_column(
        &mut self,
        timeline_cache: &mut TimelineCache,
        txn: &Transaction,
        ndb: &Ndb,
        note_cache: &mut NoteCache,
        pool: &mut RelayPool,
        kind: &TimelineKind,
    ) -> Option<TimelineOpenResult> {
        self.columns
            .push(Column::new(vec![Route::timeline(kind.to_owned())]));
        timeline_cache.open(ndb, note_cache, txn, pool, kind)
    }

    pub fn new_column_picker(&mut self) {
        self.add_column(Column::new(vec![Route::AddColumn(
            crate::ui::add_column::AddColumnRoute::Base,
        )]));
    }

    pub fn insert_intermediary_routes(
        &mut self,
        timeline_cache: &mut TimelineCache,
        intermediary_routes: Vec<IntermediaryRoute>,
    ) {
        let routes = intermediary_routes
            .into_iter()
            .map(|r| match r {
                IntermediaryRoute::Timeline(timeline) => {
                    let route = Route::timeline(timeline.kind.clone());
                    timeline_cache
                        .timelines
                        .insert(timeline.kind.clone(), timeline);
                    route
                }
                IntermediaryRoute::Route(route) => route,
            })
            .collect();

        self.columns.push(Column::new(routes));
    }

    pub fn add_column_at(&mut self, column: Column, index: u32) {
        self.columns.insert(index as usize, column);
    }

    pub fn add_column(&mut self, column: Column) {
        self.columns.push(column);
    }

    pub fn columns_mut(&mut self) -> &mut Vec<Column> {
        &mut self.columns
    }

    pub fn num_columns(&self) -> usize {
        self.columns.len()
    }

    // Get the first router in the columns if there are columns present.
    // Otherwise, create a new column picker and return the router
    pub fn get_first_router(&mut self) -> &mut Router<Route> {
        if self.columns.is_empty() {
            self.new_column_picker();
        }
        self.columns[0].router_mut()
    }

    pub fn column(&self, ind: usize) -> &Column {
        &self.columns[ind]
    }

    pub fn columns(&self) -> &[Column] {
        &self.columns
    }

    pub fn selected(&mut self) -> &mut Column {
        &mut self.columns[self.selected as usize]
    }

    pub fn column_mut(&mut self, ind: usize) -> &mut Column {
        &mut self.columns[ind]
    }

    pub fn select_down(&mut self) {
        warn!("todo: implement select_down");
    }

    pub fn select_up(&mut self) {
        warn!("todo: implement select_up");
    }

    pub fn select_left(&mut self) {
        if self.selected - 1 < 0 {
            return;
        }
        self.selected -= 1;
    }

    pub fn select_right(&mut self) {
        if self.selected + 1 >= self.columns.len() as i32 {
            return;
        }
        self.selected += 1;
    }

    #[must_use = "you must call timeline_cache.pop() for each returned value"]
    pub fn delete_column(&mut self, index: usize) -> Vec<TimelineKind> {
        let mut kinds_to_pop: Vec<TimelineKind> = vec![];
        for route in self.columns[index].router().routes() {
            if let Route::Timeline(kind) = route {
                kinds_to_pop.push(kind.clone());
            }
        }

        self.columns.remove(index);

        if self.columns.is_empty() {
            self.new_column_picker();
        }

        kinds_to_pop
    }

    pub fn move_col(&mut self, from_index: usize, to_index: usize) {
        if from_index == to_index
            || from_index >= self.columns.len()
            || to_index >= self.columns.len()
        {
            return;
        }

        self.columns.swap(from_index, to_index);
    }
}

pub enum IntermediaryRoute {
    Timeline(Timeline),
    Route(Route),
}

pub enum ColumnsAction {
    Switch(usize, usize), // from Switch.0 to Switch.1,
    Remove(usize),
}
