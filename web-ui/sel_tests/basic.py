"""TODO: these tests need to be better designed."""
from __future__ import print_function, absolute_import

import os
import time
import unittest

from selenium import webdriver
from selenium.webdriver.common.keys import Keys
from selenium.webdriver.common.by import By
from selenium.common.exceptions import NoSuchElementException
from selenium.webdriver.support.ui import WebDriverWait
from selenium.webdriver.support import expected_conditions as EC

from . import helpers
from . import artifact

LIST_VIEW_ID = "list_view"
EDIT_VIEW_ID = "edit_view"
LIST_ID = "list"

EXAMPLE_PROJ = "web-ui/sel_tests/ex_proj"


class TestStuff(unittest.TestCase):
    """TODO: this is basically a proving ground for how to write tests in
    selenium.

    It is NOT well designed (yet).

    """

    def setUp(self):
        self.driver = webdriver.Firefox()

    # pylint: disable=too-many-statements
    def test_edit_text(self):
        """Test editing a text field."""
        driver = self.driver

        art = artifact.Artifact(EXAMPLE_PROJ)
        with art as url:
            driver.get(url)
            name = "SPC-LAYOUT"

            elem = helpers.find_id(driver, name, 10)
            assert driver.find_element_by_id(
                LIST_VIEW_ID), "we are in list view"
            elem.click()

            # start editing
            edit = helpers.find_id(driver, "edit", 2)
            edit.click()

            # edit the text
            initial_text = "This is literally just a partof REQ-layout"
            elem = helpers.find_id(driver, "ed_raw_text_" + name, 1)
            assert elem.text == initial_text
            elem.send_keys(Keys.LEFT_CONTROL, "a")
            expected = "this is testing that you can edit"
            elem.send_keys(expected)
            time.sleep(0.1)
            assert elem.text == expected

            # undo the edit
            elem = driver.find_element_by_id("cancel_edit")
            elem.click()
            time.sleep(0.5)
            helpers.find_id(driver, "rd_select_text_raw", 2).click()
            elem = helpers.find_id(driver, "rd_raw_text_" + name, 2)
            assert elem.text == initial_text

            # edit again, save this time
            edit = driver.find_element_by_id("edit")
            edit.click()
            elem = helpers.find_id(driver, "ed_raw_text_" + name, 2)
            assert elem.text == initial_text
            elem.send_keys(Keys.LEFT_CONTROL, "a")
            elem.send_keys(expected)
            start = time.time()
            while elem.text != expected:
                time.sleep(0.2)
                assert time.time() - start < 2, "timeout"

            elem = driver.find_element_by_id("save")
            elem.click()
            time.sleep(0.5)
            driver.refresh()
            helpers.find_id(driver, "rd_select_text_raw", 5).click()
            assert helpers.find_id(
                driver, "rd_raw_text_" + name, 2).text == expected

            # make sure the file itself has changed
            with open(os.path.join(art.tmp_proj, "design/purpose.toml")) as purpose:
                file_text = purpose.read()
            assert expected in file_text

            # navigate away and then back... make sure it's still changed
            driver.find_element_by_id("list").click()
            helpers.find_id(driver, name, 2).click()
            assert helpers.find_id(
                driver, "rd_raw_text_" + name, 1).text == expected

            # restart the server and make sure it's still changed
            assert url == art.restart(), "urls not equal"
            time.sleep(0.5)
            driver.get(url)
            helpers.find_id(driver, name, 5).click()
            helpers.find_id(driver, "rd_select_text_raw", 5).click()
            assert helpers.find_id(
                driver, "rd_raw_text_" + name, 1).text == expected

    def test_req(self):
        """navigate to REQ and check that it is valid."""
        expected_parts = sorted(["REQ-purpose", "REQ-layout"])
        expected_partof = sorted([])

        driver = self.driver
        with artifact.Artifact(EXAMPLE_PROJ) as url:
            driver.get(url)
            name = "REQ"
            elem = helpers.find_id(driver, name, 10)
            assert driver.find_element_by_id(
                LIST_VIEW_ID), "we are in list view"
            elem.click()

            # make sure the header looks good
            elem = helpers.find_id(driver, "rd_ehead", 5)
            assert driver.find_element_by_id(
                EDIT_VIEW_ID), "we are in edit view"
            assert elem.text == name

            rd_parts = "rd_parts_" + name
            rd_partof = "rd_partof_" + name

            # make sure partof and parts are correct
            parts_list = driver.find_element_by_id(rd_parts)
            assert expected_parts == helpers.get_items(parts_list)
            partof_list = driver.find_element_by_id(rd_partof)
            assert expected_partof == helpers.get_items(partof_list)

            driver.find_element_by_id(LIST_ID).click()
            assert WebDriverWait(driver, 5).until(
                EC.presence_of_element_located((By.ID, name)))

            # parts are open by default, partof isn't
            assert driver.find_element_by_id(rd_parts)
            with self.assertRaises(NoSuchElementException):
                driver.find_element_by_id(rd_partof)

            # open columns and assert
            parts_list = driver.find_element_by_id("rd_parts_" + name)
            assert expected_parts == helpers.get_items(parts_list)

            driver.find_element_by_id("select_col_partof").click()
            partof_list = helpers.find_id(driver, rd_partof, 1)
            assert expected_partof == helpers.get_items(partof_list)

            # now close columns and assert
            driver.find_element_by_id("select_col_parts").click()
            WebDriverWait(driver, 1).until(
                EC.invisibility_of_element_located((By.ID, rd_parts)))

            driver.find_element_by_id("select_col_partof").click()
            WebDriverWait(driver, 1).until(
                EC.invisibility_of_element_located((By.ID, rd_partof)))

    def tearDown(self):
        self.driver.quit()

# if __name__ == "__main__":
#     unittest.main()
