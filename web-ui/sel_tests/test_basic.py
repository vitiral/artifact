"""TODO: these tests need to be better designed."""
from __future__ import print_function, absolute_import

import os
import time
import unittest


from selenium import webdriver
from selenium.common import exceptions
from selenium.webdriver.common.by import By
from selenium.webdriver.support.ui import WebDriverWait
from selenium.webdriver.support import expected_conditions as EC

from . import cmds
from . import webapp

CREATE = "CREATE"

LIST_VIEW_ID = "list_view"
EDIT_VIEW_ID = "edit_view"
LIST_ID = "list"

EXAMPLE_PROJ = "web-ui/sel_tests/ex_proj"


def artifact_url(base, name):
    """get the artifact-edit url."""
    return "{}/#artifacts/{}".format(base, name)


UPDATE_LOG_FMT = "Artifact Update Successful: {}".format
CREATE_LOG_FMT = "Artifact Creation Successful: {}".format


def setup_phantom():
    """setup phantom and return the bin handler and the driver."""
    phantom = cmds.Phantom()
    phantom.start()
    driver = webdriver.PhantomJS(service_log_path=os.path.devnull)
    return (phantom, driver)


class TestStuff(unittest.TestCase):
    """TODO: this is basically a proving ground for how to write tests in
    selenium.

    It is NOT well designed (yet).

    """
    @classmethod
    def setUpClass(cls):
        cls.phantom, driver = setup_phantom()
        # driver = webdriver.Firefox()
        cls.app = webapp.App(driver)

    @classmethod
    def tearDownClass(cls):
        app = getattr(cls, 'app', None)
        if app:
            app.quit()

        phantom = getattr(cls, 'phantom', None)
        if phantom:
            phantom.stop()

    def test_req(self):
        """navigate to REQ and check that it is valid."""
        expected_parts = sorted(["REQ-purpose", "REQ-layout"])
        expected_partof = sorted([])

        app = self.app
        F = webapp.Fields

        with cmds.Artifact(EXAMPLE_PROJ) as url:
            app.driver.get(url)
            name = "REQ"
            app.assert_list_view(timeout=10)
            app.goto_artifact(name, timeout=5)

            # make sure all values look good
            app.assert_read_view(timeout=5)
            app.get_value(name, F.name, timeout=2)
            assert app.get_items(name, F.parts) == expected_parts
            assert app.get_items(name, F.partof) == expected_partof

            # go back to list and assert values
            app.goto_list()
            app.assert_list_view(timeout=2)

            # parts are open by default, partof isn't
            assert app.get_items(name, F.parts, timeout=2) == expected_parts
            with self.assertRaises(exceptions.NoSuchElementException):
                app.get_items(name, F.partof)

            app.open_column(F.partof)
            assert app.get_items(name, F.partof, timeout=2) == expected_partof

            # now close columns and assert
            app.close_column(F.parts)
            WebDriverWait(app.driver, 1).until(
                EC.invisibility_of_element_located(
                    (By.ID, webapp.field_id(name, F.parts))))
            app.close_column(F.partof)
            WebDriverWait(app.driver, 1).until(
                EC.invisibility_of_element_located(
                    (By.ID, webapp.field_id(name, F.partof))))

    def test_edit_text(self):
        """Test editing a text field."""
        app = self.app
        F = webapp.Fields

        art = cmds.Artifact(EXAMPLE_PROJ)
        with art as url:
            app.driver.get(url)
            name = "SPC-LAYOUT"

            app.assert_list_view(timeout=10)
            app.goto_artifact(name, timeout=5)

            # do editing
            def assert_initial(edit, timeout=None):
                """assert that the text field has the initial text."""
                assert (app.get_value(name, F.raw_text, edit=edit, timeout=timeout)
                        == initial_text)
            initial_text = "This is literally just a partof REQ-layout"
            expected = "this is testing that you can edit"

            app.assert_read_view(timeout=2)
            app.select_text(F.raw_text, timeout=2)
            assert_initial(edit=False, timeout=1)
            app.start_edit(timeout=1)
            app.assert_edit_view(timeout=1)
            assert_initial(edit=True, timeout=1)
            app.set_value(name, F.raw_text, expected)
            # editing does not change read
            assert_initial(edit=False, timeout=1)

            # undo the edit
            app.cancel_edit()
            assert_initial(edit=False, timeout=1)
            assert app.get_value(name, F.raw_text, timeout=2) == initial_text

            # edit again, save this time
            app.start_edit(timeout=1)
            assert_initial(edit=True, timeout=1)
            app.set_value(name, F.raw_text, expected)
            app.save_edit()
            app.ack_log(0, UPDATE_LOG_FMT("SPC-layout"), timeout=1)
            time.sleep(0.5)
            app.driver.refresh()
            app.select_text(F.raw_text, timeout=2)
            assert app.get_value(name, F.raw_text, timeout=1) == expected

            # make sure the file itself has changed as expected
            toml_arts = art.get_artifacts("design/purpose.toml")
            assert toml_arts["SPC-layout"].text == expected

            # navigate away and then back... make sure it's still changed
            app.goto_list()
            app.goto_artifact(name, timeout=2)
            assert app.get_value(name, F.raw_text, timeout=2) == expected

            # restart the server and make sure it's still changed
            assert url == art.restart(), "urls not equal"
            time.sleep(0.5)
            app.driver.get(url)  # refreshes app
            app.goto_artifact(name, timeout=5)
            app.select_text(F.raw_text, timeout=2)
            assert app.get_value(name, F.raw_text, timeout=2) == expected

    def test_edit_partof(self):
        """Test editing a text field."""
        app = self.app
        F = webapp.Fields

        art = cmds.Artifact(EXAMPLE_PROJ)
        with art as url:
            app.driver.get(url)
            name = "SPC-LAYOUT"
            expected_partof = sorted(["REQ-layout", "SPC", "SPC-alone"])

            app.assert_list_view(timeout=10)
            app.goto_artifact(name, timeout=5)
            app.assert_read_view(timeout=2)
            app.start_edit(timeout=1)
            app.assert_edit_view(timeout=2)
            # do some wiggling, note that each call auto-validates
            app.add_partof(name, "SPC-alone", timeout=2)
            app.set_partof(name, "SPC-alone", "REQ-purpose")
            app.set_partof(name, "REQ-purpose", "SPC-alone")
            app.remove_partof(name, "SPC-alone")

            # finally add it and save
            app.add_partof(name, "SPC-alone")
            app.save_edit()
            app.ack_log(0, UPDATE_LOG_FMT("SPC-layout"), timeout=1)
            app.assert_read_view()
            assert app.get_items(name, F.partof) == expected_partof

            toml_arts = art.get_artifacts("design/purpose.toml")
            assert toml_arts["SPC-layout"].partof == "SPC-alone"

    def test_edit_name(self):
        """Test that editing the name works as expected."""
        app = self.app
        F = webapp.Fields

        art = cmds.Artifact(EXAMPLE_PROJ)
        with art as url:
            app.driver.get(url)
            name = "SPC-LAYOUT"

            app.assert_list_view(timeout=10)
            app.goto_artifact(name, timeout=5)
            app.assert_read_view(timeout=2)
            assert app.driver.current_url == artifact_url(url, name.lower())
            app.start_edit(timeout=1)
            app.assert_edit_view(timeout=2)

            new_name_raw = "SPC-lay"
            new_name = new_name_raw.upper()
            app.set_value(name, F.name, new_name_raw, 2)
            app.save_edit()
            app.ack_log(0, UPDATE_LOG_FMT(new_name_raw), timeout=1)
            # assert name changed and url changed
            assert app.get_value(new_name, F.name, timeout=1) == new_name_raw
            assert app.driver.current_url == artifact_url(
                url, new_name.lower())

    def test_edit_defined(self):
        """Test that editing the defined path works as expected."""
        app = self.app
        F = webapp.Fields

        art = cmds.Artifact(EXAMPLE_PROJ)
        with art as url:
            app.driver.get(url)
            name_raw = "SPC-alone"
            name = name_raw.upper()
            initial_path = "design/alone.toml"
            new_path = "design/purpose.toml"

            app.assert_list_view(timeout=10)
            app.goto_artifact(name, timeout=5)
            app.assert_read_view(timeout=2)
            assert app.driver.current_url == artifact_url(url, name.lower())
            assert app.get_value(name, F.def_at) == initial_path
            app.start_edit(timeout=1)
            app.assert_edit_view(timeout=2)

            app.set_defined(name, new_path)
            app.save_edit()
            app.ack_log(0, UPDATE_LOG_FMT(name_raw), timeout=1)
            assert app.get_value(name, F.def_at) == new_path
            assert name_raw not in art.get_artifacts(initial_path)
            assert name_raw in art.get_artifacts(new_path)

            art.restart()

    def test_create(self):
        """Test editing a text field."""
        app = self.app
        F = webapp.Fields

        art = cmds.Artifact(EXAMPLE_PROJ)
        with art as url:
            app.driver.get(url)
            name_raw = "spc-created"
            name = name_raw.upper()
            expected = "I created this and it rocks\nwhoo!"
            expected_partof = sorted(["REQ-purpose", "SPC"])
            defined = "design/purpose.toml"

            app.assert_list_view(timeout=10)
            app.goto_create()
            assert app.driver.current_url == url + "/#create"

            assert app.get_attr("save", "disabled", timeout=2) == 'true'
            app.set_value(CREATE, F.name, name_raw)
            app.set_value(CREATE, F.raw_text, expected)
            app.add_partof(CREATE, "REQ-purpose")
            app.set_defined(CREATE, defined)
            app.save_create()
            app.ack_log(0, CREATE_LOG_FMT(name_raw), timeout=1)

            # we have started a new session
            assert app.get_attr("save", "disabled", timeout=2) == 'true'

            app.goto_list()
            app.goto_artifact(name, timeout=2)
            assert app.get_value(name, F.name) == name_raw
            app.select_text(F.raw_text, timeout=2)
            assert app.get_value(name, F.raw_text) == expected
            assert app.get_items(name, F.partof) == expected_partof
            assert app.get_value(name, F.def_at) == defined
