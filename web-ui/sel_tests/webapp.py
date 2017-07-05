"""This module defines the App class, which has helper methods for navigating
around the application."""
from __future__ import print_function, unicode_literals

import time

from selenium import webdriver
from selenium.webdriver.common.keys import Keys
from selenium.webdriver.common.by import By
from selenium.common import exceptions
from selenium.webdriver.support.ui import (WebDriverWait, Select)
from selenium.webdriver.support import expected_conditions as EC


class Fields(object):
    """Available Fields/Columns."""
    name = 'name'
    parts = 'parts'
    partof = 'partof'
    text = 'text'
    raw_text = 'raw_text'
    rendered_text = 'rendered_text'
    def_at = 'def'
    done = 'done'


class Event(object):
    """Available events."""
    input = 'input'


F = Fields
E = Event


def field_id(name, field, edit=False, extra=None):
    """return the formatted field id."""
    extra = "" if extra is None else "_" + extra
    return "{}_{}_{}{}".format(_get_type(edit), field, name, extra)


def get_items(list_element):
    """Get item elements from a list element."""
    return sorted(p.text for p in list_element.find_elements_by_tag_name('li'))


class App(object):
    """Helper methods for accessing navigating the web-app and getting/setting
    values."""

    def __init__(self, driver):
        self.driver = driver
        if isinstance(driver, webdriver.PhantomJS):
            print("using phantomjs workaround for alerts")
            driver.execute_script("window.confirm = function(){return true;}")

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

    def get_attr(self, id_, attr, timeout=None):
        """Get an attribute value attached to id."""
        return self.find_id(id_, timeout).get_attribute(attr)

    def assert_no_id(self, id_, timeout=None, msg=None):
        """Assert that the id goes out of existence within timeout."""
        try:
            if timeout is None:
                try:
                    self.find_id(id_)
                    assert False, "id {} exists".format(id_)
                except exceptions.NoSuchElementException:
                    pass
            else:
                WebDriverWait(self.driver, timeout).until(
                    EC.invisibility_of_element_located(
                        (By.ID, id_)))
        except Exception as e:  # pylint: disable=broad-except
            if msg is None:
                raise
            else:
                raise AssertionError("{}: {}", e, msg)

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

    def trigger_event(self, id_, event):
        """Hack to force an event to trigger in the webapp.

        This is to get around bugs related to selenium.

        """
        js = '''
        var event = new Event("%s");
        element = document.getElementById("%s")
        element.dispatchEvent(event);
        return true;
        ''' % (event, id_)
        assert self.driver.execute_script(js)

    def accept_refresh(self, timeout=None):
        """When given the alert to stay on the page, accept."""
        self.wait_for_alert(timeout)
        try:
            self.driver.switch_to.alert.accept()
        finally:
            self.driver.switch_to.default_content()

    def alert_exists(self):
        """Return True if an alert box exists."""
        try:
            self.driver.switch_to.alert.text
        except exceptions.NoAlertPresentException:
            return False
        else:
            return True
        finally:
            self.driver.switch_to.default_content()

    def wait_for_alert(self, timeout=None):
        """wait for an alert to appear."""
        if timeout is None:
            return
        start = time.time()
        while not self.alert_exists():
            assert time.time() - start < timeout, "timeout waiting for alert"
            time.sleep(0.1)

    ################################################################################
    # Both List and Edit Views

    def get_value(self, name, field, edit=False, timeout=None):
        """get the value from a field."""
        return self.find_id(field_id(name, field, edit), timeout).text

    def set_value(self, name, field, value, timeout=None):
        """Set the value in an editable field and assert it gets set."""
        elem = self.find_id(field_id(name, field, edit=True), timeout)
        time.sleep(0.1)
        elem.send_keys(Keys.LEFT_CONTROL, "a")
        time.sleep(0.2)
        elem.send_keys(value)
        start = time.time()
        while elem.get_attribute('value') != value:
            time.sleep(0.2)
            assert time.time() - start < 3, "timeout"

    def get_items(self, name, field, edit=False, timeout=None):
        """Get the items of artifact in a field."""
        elem = self.find_id(field_id(name, field, edit), timeout)
        return get_items(elem)

    def ack_log(self, index, text, timeout=None):
        """Assert that logs[index].text == text and ack."""
        id_ = "log_text_{}".format(index)
        assert self.find_id(id_, timeout).text == text
        self.find_id("ack_log_{}".format(index)).click()

    def goto_create(self, timeout=None):
        """Goto the create page."""
        self.find_id("create", timeout).click()
        self.assert_create_view(timeout=2)

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
        self.assert_no_id("th_" + column, timeout=2, msg="column already open")
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

    def assert_read_view(self, timeout=None):
        """assert we are in the read view."""
        assert self.find_id('read_view', timeout), 'not in read view'

    def assert_edit_view(self, timeout=None):
        """assert we are in the edit view."""
        assert self.find_id('edit_view', timeout), 'not in edit view'

    def assert_create_view(self, timeout=None):
        """assert we are in the create view."""
        assert self.find_id('create_view', timeout), 'not in create view'

    def goto_list(self, timeout=None):
        """Go to the list view while in the edit view."""
        self.find_id("list", timeout).click()

    def select_text(self, field, edit=False, timeout=None):
        """select a specific kind of text field."""
        assert field in {F.raw_text, F.rendered_text}
        self.find_id("{}_select_{}".format(
            _get_type(edit), field), timeout).click()

    def add_partof(self, name, partof, timeout=None):
        """Add a partof item to artifact name in the edit view."""
        select_id = "add_partof_" + name
        select = self.find_id(select_id, timeout)
        Select(select).select_by_visible_text("  " + partof)

        # validate that it appears
        id_ = field_id(name, F.partof, edit=True, extra=partof.upper())
        assert self.find_id(id_, timeout=1)

    def set_partof(self, name, from_partof, to_partof, timeout=None):
        """Change a partof value to a new value in the edit view."""
        select_id = "select_partof_{}_{}".format(name, from_partof.upper())
        select = self.find_id(select_id, timeout)
        Select(select).select_by_visible_text("  " + to_partof)
        # validate that it appears
        assert self.find_id(field_id(name, F.partof, edit=True, extra=to_partof.upper()),
                            timeout=2)

    def remove_partof(self, name, partof):
        """Remove a partof in the edit view."""
        self.find_id("delete_partof_{}_{}".format(
            name, partof.upper())).click()
        self.assert_no_id(field_id(name, F.partof, edit=True, extra=partof),
                          timeout=1)

    def set_defined(self, name, value, timeout=None):
        """Set the defined parameter."""
        select = self.find_id(field_id(name, F.def_at, edit=True), timeout)
        Select(select).select_by_visible_text(value)
        assert select.get_attribute('value') == value

    def start_edit(self, timeout=None):
        """Start edit and wait for it to start."""
        self.find_id("edit", timeout).click()
        assert self.find_id("cancel_edit", 1)

    def save_edit(self, timeout=None):
        """Save an editing session and wait until it is registered."""
        self.find_id("save", timeout).click()
        assert self.find_id("edit", 10)

    def save_create(self, timeout=None):
        """Save while in the create page."""
        self.find_id("save", timeout).click()

    def delete(self, timeout=None):
        """Delete the current artifact."""
        self.find_id("delete", timeout).click()
        self.assert_list_view(timeout=5)

    def cancel_edit(self, timeout=None):
        """Cancel edit and wait for it to be canceled."""
        self.find_id("cancel_edit", timeout).click()
        self.find_id("edit", 1)


def _get_type(edit):
    """return ed (edit) or rd (read) field type."""
    return "ed" if edit else "rd"
