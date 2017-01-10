
import time
import unittest

from selenium import webdriver
from selenium.webdriver.common.keys import Keys
from selenium.webdriver.common.by import By
from selenium.webdriver.support.ui import WebDriverWait
from selenium.webdriver.support import expected_conditions as EC


class BasicTest(unittest.TestCase):

    def setUp(self):
        self.driver = webdriver.Firefox()
        self.driver.get('http://127.0.0.1:4000')

    def test_req(self):
        ''' navigate to REQ and check that it is valid '''
        driver = self.driver
        name = "REQ"
        elem = driver.find_element_by_id(name)
        elem.click()

        expected_parts = ["REQ-purpose", "REQ-layout"]
        expected_partof = []

        expected_parts = sorted(expected_parts)
        expected_partof = sorted(expected_partof)

        # make sure the header looks good
        elem = WebDriverWait(driver, 10).until(
            EC.presence_of_element_located((By.ID, "ehead")))
        assert elem.text == name

        # make sure partof and parts are correct
        partof_list = driver.find_element_by_id("partof_" + name)
        parts_list = driver.find_element_by_id("parts_" + name)

        assert expected_parts == sorted(p.text for p in parts_list.find_elements_by_tag_name('li'))
        assert expected_partof == sorted(p.text for p in partof_list.find_elements_by_tag_name('li'))

        # self.assertIn("Python", driver.title)
        # elem = driver.find_element_by_name("q")
        # elem.send_keys("pycon")
        # elem.send_keys(Keys.RETURN)
        # assert "No results found." not in driver.page_source

    def tearDown(self):
        self.driver.quit()

# if __name__ == "__main__":
#     unittest.main()
