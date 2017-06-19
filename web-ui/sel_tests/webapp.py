"""This module defines the App class, which has helper methods for navigating
around the application."""

import time

from selenium.webdriver.common.keys import Keys
from selenium.webdriver.common.by import By
# from selenium.common import exceptions import NoSuchElementException
from selenium.common import exceptions
from selenium.webdriver.support.ui import WebDriverWait
from selenium.webdriver.support import expected_conditions as EC


class Fields(object):
    """Available Fields/Columns."""
    name = 'name'
    parts = 'parts'
    partof = 'partof'
    text = 'text'
    raw_text = 'raw_text'
    rendered_text = 'rendered_text'
    def_at = 'def-at'
    done = 'done'


def field_id(name, field, edit=False):
    """return the formatted field id."""
    return "{}_{}_{}".format(_get_type(edit), field, name)


def get_items(list_element):
    """Get item elements from a list element."""
    return sorted(p.text for p in list_element.find_elements_by_tag_name('li'))


class App(object):
    """Helper methods for accessing navigating the web-app and getting/setting
    values."""

    def __init__(self, driver):
        self.driver = driver

    def quit(self):
        """quit the app."""
        self.driver.quit()

    # General Purpose Methods

    def find_id(self, id_, timeout=None):
        """find an element of id with a timeout.

        If timeout=None, just use driver.find_element_by_id

        """
        if timeout is None:
            return self.driver.find_element_by_id(id_)
        return WebDriverWait(self.driver, timeout).until(
            EC.presence_of_element_located((By.ID, id_)))

    def get_ids(self, must_contain=None):
        """Return a list of all ids in the page.

        Mostly for debugging.

        """
        ids = self.driver.find_elements_by_xpath('//*[@id]')
        out = []
        for i in ids:
            attr = i.get_attribute('id')
            if must_contain is not None and must_contain not in attr:
                continue
            out.append(attr)
        return out

    ################################################################################
    # Both List and Edit Views

    def get_value(self, name, field, edit=False, timeout=None):
        """get the value from a field."""
        return self.find_id(field_id(name, field, edit), timeout).text

    def set_value(self, name, field, value, timeout=None):
        """Set the value in an editable field and assert it gets set."""
        elem = self.find_id(field_id(name, field, edit=True), timeout)
        elem.send_keys(Keys.LEFT_CONTROL, "a")
        elem.send_keys(value)
        start = time.time()
        while elem.text != value:
            time.sleep(0.2)
            assert time.time() - start < 2, "timeout"

    def get_items(self, name, field, edit=False, timeout=None):
        """Get the items of artifact in a field."""
        elem = self.find_id(field_id(name, field, edit), timeout)
        return get_items(elem)

    ################################################################################
    # List View Helpers

    def assert_list_view(self, timeout=None):
        """assert that we are in the list view."""
        assert self.find_id('list_view', timeout), 'not in list view'

    def goto_artifact(self, name, timeout=None):
        """Goto an artifact while in the list view."""
        self.find_id(name, timeout).click()

    def open_column(self, column):
        """Open the requested column. If column is already open raise an error.

        This should only be run on a loaded page

        """
        try:
            self.find_id("th_" + column)
            assert False, "column already open"
        except exceptions.NoSuchElementException:
            pass
        self.find_id("select_col_" + column).click()

    def close_column(self, column):
        """Close the requested column. If column is already open raise an
        error.

        This should only be run on a loaded page

        """
        # assert column is open
        assert self.find_id("th_" + column), "column is not open"
        self.find_id("select_col_" + column).click()

    def search(self, pattern, timeout=None):
        """enter text into the search bar, clearing what was there."""
        elem = self.find_id("search_input", timeout)
        elem.send_keys(Keys.LEFT_CONTROL, "a")
        elem.send_keys(pattern)
        time.sleep(0.1)  # sleep seems to improve stability
        assert elem.text == pattern, "search input didn't register"

    ################################################################################
    # Edit View Helpers

    def assert_edit_view(self, timeout=None):
        """assert we are in the edit view."""
        assert self.find_id('edit_view', timeout), 'not in edit view'

    def goto_list(self, timeout=None):
        """Go to the list view while in the edit view."""
        self.find_id("list", timeout).click()

    def select_text(self, field, edit=False, timeout=None):
        """select a specific kind of text field."""
        assert field in {Fields.raw_text, Fields.rendered_text}
        self.find_id("{}_select_{}".format(
            _get_type(edit), field), timeout).click()

    def start_edit(self, timeout=None):
        """Start edit and wait for it to start."""
        self.find_id("edit", timeout).click()
        assert self.find_id("cancel_edit", 1)

    def save_edit(self, timeout=None):
        """Save an editing session and wait until it is registered."""
        self.find_id("save", timeout).click()
        assert self.find_id("edit", 3)

    def cancel_edit(self, timeout=None):
        """Cancel edit and wait for it to be canceled."""
        self.find_id("cancel_edit", timeout).click()
        self.find_id("edit", 1)


def _get_type(edit):
    """return ed (edit) or rd (read) field type."""
    return "ed" if edit else "rd"
